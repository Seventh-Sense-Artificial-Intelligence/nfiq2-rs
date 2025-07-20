use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_ushort};
use std::ptr;

use crate::errors::Nfiq2Error;

#[repr(C)]
pub(crate) struct Nfiq2ResultsT {
    score: c_uint,
    actionable_count: c_uint,
    actionable_ids: *const *const c_char,
    actionable_values: *mut f64,
    feature_count: c_uint,
    feature_ids: *const *const c_char,
    feature_values: *mut f64,
}

/// Opaque C++ wrapper handle
#[repr(C)]
pub struct Nfiq2WrapperOpaque {
    _private: [u8; 0],
}

// FFI imports
extern "C" {
    fn nfiq2wrapper_create() -> *mut Nfiq2WrapperOpaque;
    fn nfiq2wrapper_destroy(ctx: *mut Nfiq2WrapperOpaque);

    fn nfiq2wrapper_compute(
        ctx: *mut Nfiq2WrapperOpaque,
        data: *const c_uchar,
        size: c_uint,
        cols: c_uint,
        rows: c_uint,
        ppi: c_ushort,
        out: *mut Nfiq2ResultsT,
    ) -> c_int;

    fn nfiq2wrapper_free_results(out: *mut Nfiq2ResultsT);
}

#[derive(Debug, uniffi::Record)]
pub struct Nfiq2Value {
    name: String,
    value: f64,
}

/// Safe Rust view of the results
#[derive(Debug, uniffi::Record)]
pub struct Nfiq2Result {
    pub score: u32,
    pub actionable: Vec<Nfiq2Value>,
    pub features: Vec<Nfiq2Value>,
}

/// The high‐level Rust handle
#[derive(Debug, Clone, uniffi::Object)]
pub struct Nfiq2 {
    ctx: *mut Nfiq2WrapperOpaque,
}

unsafe impl Send for Nfiq2 {}
unsafe impl Sync for Nfiq2 {}

/// Construct a new wrapper, or Err if allocation/initialization fails.
#[uniffi::export]
pub fn create_nfiq2() -> Result<Nfiq2, Nfiq2Error> {
    let ptr = unsafe { nfiq2wrapper_create() };
    if ptr.is_null() {
        Err(Nfiq2Error::CreateFailed)
    } else {
        Ok(Nfiq2 { ctx: ptr })
    }
}

#[uniffi::export]
impl Nfiq2 {
    /// Compute quality. Mirrors your C API.
    pub fn compute(&self, image_bytes: &[u8]) -> Result<Nfiq2Result, Nfiq2Error> {
        if self.ctx.is_null() {
            return Err(Nfiq2Error::NullContext);
        }

        // load the image from bytes
        let image =
            image::load_from_memory(image_bytes).map_err(|_| Nfiq2Error::ComputeFailed(-1))?;

        // convert to grayscale and get dimensions
        let image = image.to_luma8();

        let (cols, rows) = image.dimensions();
        let ppi = 500; // hardcoded PPI, can be adjusted as needed

        // zero the C struct
        let mut raw: Nfiq2ResultsT = unsafe { std::mem::zeroed() };

        let size = image.len() as c_uint;

        let rc = unsafe {
            nfiq2wrapper_compute(
                self.ctx,
                image.as_ptr(),
                size as c_uint,
                cols as c_uint,
                rows as c_uint,
                ppi as c_ushort,
                &mut raw,
            )
        };
        if rc != 0 {
            // free any partial allocations before returning
            unsafe { nfiq2wrapper_free_results(&mut raw) };
            return Err(Nfiq2Error::ComputeFailed(rc));
        }

        // helper to turn C arrays into Vec<(String,f64)>
        unsafe fn collect_pairs(
            ids_ptr: *const *const c_char,
            vals_ptr: *const f64,
            count: usize,
        ) -> Result<Vec<Nfiq2Value>, Nfiq2Error> {
            let mut out = Vec::with_capacity(count);
            let id_slice = std::slice::from_raw_parts(ids_ptr, count);
            let val_slice = std::slice::from_raw_parts(vals_ptr, count);
            for i in 0..count {
                let s = CStr::from_ptr(id_slice[i]).to_str()
                    .map_err(|_| Nfiq2Error::ComputeFailed(-1))?
                    .to_string();

                out.push(Nfiq2Value {
                    name: s,
                    value: val_slice[i],
                });
            }
            Ok(out)
        }

        let actionable_count = raw.actionable_count as usize;
        let feature_count = raw.feature_count as usize;

        // collect actionable + features
        let actionable = unsafe {
            collect_pairs(
                raw.actionable_ids,
                raw.actionable_values as *const f64,
                actionable_count,
            )?
        };
        let features = unsafe {
            collect_pairs(
                raw.feature_ids,
                raw.feature_values as *const f64,
                feature_count,
            )?
        };

        let score = raw.score;

        // free C allocations
        unsafe { nfiq2wrapper_free_results(&mut raw) };

        Ok(Nfiq2Result {
            score,
            actionable,
            features,
        })
    }
}

impl Drop for Nfiq2 {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { nfiq2wrapper_destroy(self.ctx) };
            self.ctx = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfiq2() {
        // construct wrapper
        let nfiq = create_nfiq2().expect("failed to create wrapper");

        let expected_scores = vec![54, 45, 53, 52, 57];
        let input_images = vec![
            "ext/NFIQ2-2.3.0/examples/images/SFinGe_Test01.pgm",
            "ext/NFIQ2-2.3.0/examples/images/SFinGe_Test02.pgm",
            "ext/NFIQ2-2.3.0/examples/images/SFinGe_Test03.pgm",
            "ext/NFIQ2-2.3.0/examples/images/SFinGe_Test04.pgm",
            "ext/NFIQ2-2.3.0/examples/images/SFinGe_Test05.pgm",
        ];

        for (i, img_path) in input_images.iter().enumerate() {
            // load test image bytes
            let img_bytes = std::fs::read(img_path).expect("failed to read test image");

            // call compute
            let res = nfiq.compute(&img_bytes).expect("compute failed");

            // check score
            assert_eq!(res.score, expected_scores[i]);
        }
    }
}
