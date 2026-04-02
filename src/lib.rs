//! # Basis Set Exchange in Rust (bse-rs)
//!
//! A Rust library for retrieving, manipulating, and converting Gaussian-type
//! orbital (GTO) basis sets for computational chemistry.
//!
//! This is a Rust reimplementation of the Python [Basis Set Exchange](
//! https://github.com/MolSSI-BSE/basis_set_exchange/) library, providing
//! APIs compatible with the MolSSI Basis Set Exchange project.
//!
//! ## Features
//!
//! - **Basis set retrieval**: Get basis sets by name with element filtering
//! - **Format conversion**: Export to 20+ quantum chemistry formats (NWChem,
//!   Gaussian, ORCA, etc.)
//! - **Basis set manipulation**: Uncontract, optimize, and augment basis sets
//! - **Metadata queries**: List available basis sets, families, and roles
//! - **Reader support**: Parse basis sets from various input formats
//!
//! ## Quick Start
//!
//! ```rust
//! use bse::prelude::*;
//!
//! // Get a basis set as a structured object
//! let args = BseGetBasisArgsBuilder::default()
//!     .elements("H, C-O".to_string())
//!     .build()
//!     .unwrap();
//! let basis = get_basis("cc-pVTZ", args);
//! println!("Basis: {}", basis.name);
//!
//! // Get formatted output for a specific software
//! let args = BseGetBasisArgsBuilder::default()
//!     .elements("H, O".to_string())
//!     .header(true)
//!     .build()
//!     .unwrap();
//! let output = get_formatted_basis("sto-3g", "nwchem", args);
//! println!("{}", output);
//! ```
//!
//! ## Data Directory Setup
//!
//! This crate requires basis set data from the Python BSE project. Set the
//! `BSE_DATA_DIR` environment variable:
//!
//! ```bash
//! export BSE_DATA_DIR=/path/to/basis_set_exchange/basis_set_exchange/data
//! ```
//!
//! Alternatively, call [`specify_bse_data_dir`][api::specify_bse_data_dir]
//! at runtime, or the library will attempt auto-detection.
//!
//! ## References
//!
//! - [MolSSI Basis Set Exchange](https://www.basissetexchange.org)
//! - [Python BSE Repository](https://github.com/MolSSI-BSE/basis_set_exchange)

#![allow(non_snake_case)]
#![allow(clippy::needless_range_loop)]

pub mod api;
pub mod cli;
pub mod compose;
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
