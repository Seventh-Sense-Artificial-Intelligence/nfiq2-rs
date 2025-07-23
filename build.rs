use std::{
    env, fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

fn android_abi_from_target(target: &str) -> Option<&'static str> {
    if target.contains("aarch64") {
        Some("arm64-v8a")
    } else if target.contains("armv7") {
        Some("armeabi-v7a")
    } else if target.contains("x86_64") {
        Some("x86_64")
    } else if target.contains("i686") {
        Some("x86")
    } else {
        None
    }
}

fn copy_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    // clear out any old files
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    // recreate root
    fs::create_dir_all(target)?;

    for entry in WalkDir::new(source) {
        let entry = entry?;
        let rel_path: PathBuf = entry.path().strip_prefix(source).unwrap().into();
        let dest_path = target.join(&rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            // make sure parent dir exists
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-env-changed=CLIPPY");

    let target = env::var("TARGET").unwrap_or_default();
    let is_android = target.contains("android");
    let is_linux = target.contains("linux") && !target.contains("android");
    let is_macos = target.contains("apple") || target.contains("darwin");

    let source = Path::new("ext/opencv-4.10.0");
    let target = Path::new("ext/NFIQ2-2.3.0/opencv");
    copy_dir(source, target).expect("failed to copy OpenCV dir");

    let source = Path::new("ext/FingerJetFXOSE");
    let target = Path::new("ext/NFIQ2-2.3.0/fingerjetfxose");
    copy_dir(source, target).expect("failed to copy FingerJetFXOSE dir");

    let source = Path::new("ext/digestpp");
    let target = Path::new("ext/NFIQ2-2.3.0/digestpp");
    copy_dir(source, target).expect("failed to copy FingerJetFXOSE dir");

    let source = Path::new("ext/libbiomeval-10.0");
    let target = Path::new("ext/NFIQ2-2.3.0/libbiomeval");
    copy_dir(source, target).expect("failed to copy libbiomeval-10.0 dir");

    // ---- CMake for NFIQ2 ----
    let mut cmake = cmake::Config::new("ext/NFIQ2-2.3.0");
    cmake
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_INSTALL_PREFIX", "NFIQ2-2.3.0/install")
        .define("EMBED_RANDOM_FOREST_PARAMETERS", "ON")
        .define("EMBEDDED_RANDOM_FOREST_PARAMETER_FCT", "3");

    if is_android {
        let target = env::var("TARGET").unwrap_or_default();
        let ndk = env::var("ANDROID_NDK_ROOT").expect("ANDROID_NDK_ROOT not set");
        let abi = if target.contains("aarch64") {
            "arm64-v8a"
        } else if target.contains("armv7") {
            "armeabi-v7a"
        } else {
            panic!("Unsupported Android ABI: {}", target);
        };
        cmake.define("ANDROID_ABI", abi);
        cmake.define(
            "CMAKE_TOOLCHAIN_FILE",
            format!("{}/build/cmake/android.toolchain.cmake", ndk),
        );
        cmake.define("BUILD_NFIQ2_CLI", "OFF");
    } else {
        cmake.define("BUILD_NFIQ2_CLI", "OFF");
    }

    let dst = cmake.build();

    // Define the include and library paths for NFIQ2
    let nfiq2_include_path = dst.join("build/install_staging/nfiq2/include");
    let nfiq2_lib_path = dst.join("build/install_staging/nfiq2/lib");

    // On Android, OpenCV libraries are in a different location
    let opencv_android_lib_path = if is_android {
        let target = env::var("TARGET").unwrap_or_default();
        let abi = android_abi_from_target(&target).expect("Unsupported Android ABI");
        Some(dst.join(format!(
            "build/install_staging/nfiq2/sdk/native/staticlibs/{}",
            abi
        )))
    } else {
        None
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let opencv_lib_path = out_dir.join("build/install_staging/nfiq2/lib/opencv4/3rdparty");

    // 1) Compile the C++ FFI wrapper
    cc::Build::new()
        .cpp(true) // switch to a C++ compiler
        .flag_if_supported("-std=c++14") // or c++11/17, whichever you need
        .include(&nfiq2_include_path) // where nfiq2.hpp lives
        .include("src/cwrapper")
        .file("src/cwrapper/nfiq_wrapper.cpp") // your FFI source
        .define("NOVERBOSE", None) // you probably don’t want stdout spam
        .flag_if_supported("-w") // for GCC/Clang: suppress *all* warnings
        .compile("nfiq2_ffi"); // emits libnfiq2_ffi.a

    // 2) Link against both the wrapper and the NFIQ2 / OpenCV libs
    println!("cargo:rustc-link-lib=static=nfiq2_ffi");
    println!(
        "cargo:rustc-link-search=native={}",
        nfiq2_lib_path.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        opencv_lib_path.display()
    );

    // Include the OpenCV libs path for Android
    if let Some(opencv_android_lib_path) = opencv_android_lib_path {
        println!(
            "cargo:rustc-link-search=native={}",
            opencv_android_lib_path.display()
        );
    }

    // if you built NFIQ2 via cmake earlier in the same build.rs, you'd also:
    println!("cargo:rustc-link-lib=static=nfiq2");
    println!("cargo:rustc-link-lib=static=opencv_ml");
    println!("cargo:rustc-link-lib=static=opencv_imgcodecs");
    println!("cargo:rustc-link-lib=static=opencv_imgproc");
    println!("cargo:rustc-link-lib=static=opencv_core");
    println!("cargo:rustc-link-lib=static=FRFXLL_static");

    if is_linux {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=z");
    }

    if is_android {
        println!("cargo:rustc-link-lib=z");
    }

    // ...and link-search to wherever CMake put its .a files:

    if is_macos {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=OpenCL");
        println!("cargo:rustc-link-lib=static=tegra_hal");
        println!("cargo:rustc-link-lib=static=zlib");
    }

    println!("cargo:rerun-if-changed=src/cwrapper/nfiq_wrapper.cpp");
}
