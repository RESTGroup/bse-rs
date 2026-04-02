//! # Basis Set Exchange in Rust (bse-rs)
//!
//! 

#![allow(non_snake_case)]
#![allow(clippy::needless_range_loop)]

pub mod api;
pub mod cli;
pub mod compose;
pub mod dir_reader;
pub mod dir_writer;
pub mod error;
pub mod fields;
pub mod ints;
pub mod lut;
pub mod lut_data;
pub mod manip;
pub mod misc;
pub mod notes;
pub mod prelude;
pub mod printing;
pub mod readers;
pub mod refconverters;
pub mod references;
pub mod sort;
pub mod writers;

// Re-export commonly used items at crate root for convenience
pub use error::BseError;
pub use prelude::*;

#[cfg(feature = "remote")]
pub mod client;
