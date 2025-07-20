uniffi::setup_scaffolding!();
mod errors;
mod ffi;

pub use errors::Nfiq2Error;
pub use ffi::{create_nfiq2, Nfiq2, Nfiq2Result, Nfiq2Value};
