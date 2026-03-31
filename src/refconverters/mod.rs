//! Reference/citation format converters.
//!
//! This module provides converters for exporting basis set references/citations
//! in various formats including BibTeX, RIS, EndNote, plain text, and JSON.
//!
//! # Supported Formats
//!
//! - `bib` - BibTeX format (for LaTeX bibliographies)
//! - `ris` - RIS format (for Reference Manager, EndNote import)
//! - `endnote` - EndNote format (.enw files)
//! - `txt` - Plain text format (human-readable)
//! - `json` - JSON format (raw data)
//!
//! # Example
//!
//! ```rust
//! use bse::prelude::*;
//!
//! // Get reference data for a basis set
//! let basis = get_basis("cc-pVTZ", BseGetBasisArgs::default());
//! let ref_data = get_references("cc-pVTZ", None);
//!
//! // Convert to BibTeX format using the convenience function
//! let bib_str = get_references_formatted("cc-pVTZ", None, None, "bib");
//! println!("{}", bib_str);
//! ```

mod bib;
mod common;
mod convert;
mod endnote;
mod ris;

pub use bib::write_bib;
pub use common::get_library_citation;
pub use convert::{convert_references, get_reference_format_extension, get_reference_formats};
pub use endnote::write_endnote;
pub use ris::write_ris;
