#![doc = include_str!("../README.md")]
uniffi::setup_scaffolding!();
mod api;
mod errors;
mod ffi;

pub use api::{create_nfiq2, Nfiq2, Nfiq2Result, Nfiq2Value};
pub use errors::Nfiq2Error;
