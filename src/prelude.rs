//! Common imports for users and developers of the bse crate.
//!
//! This module re-exports the most commonly used items from the crate,
//! allowing users to import everything with a single `use bse::prelude::*;`.
//!
//! # For Users
//!
//! The following items are re-exported for general use:
//!
//! ## Main API Functions
//! - [`get_basis`], [`get_basis_f`] - Retrieve a basis set by name
//! - [`get_formatted_basis`], [`get_formatted_basis_f`] - Get formatted output
//!   for specific software
//! - [`get_metadata`], [`get_metadata_f`] - Get metadata for all basis sets
//! - [`get_all_basis_names`], [`get_all_basis_names_f`] - List all basis set
//!   names
//! - [`get_families`], [`get_families_f`] - List all basis set families
//! - [`get_roles`] - Get available basis set roles
//! - [`lookup_basis_by_role`], [`lookup_basis_by_role_f`] - Find auxiliary
//!   basis sets
//! - [`filter_basis_sets`], [`filter_basis_sets_f`] - Filter basis sets by
//!   criteria
//! - [`get_formats`] - Get available output formats
//!
//! ## Data Structures
//! - [`BseBasis`] - Complete basis set information
//! - [`BseBasisElement`] - Per-element basis data
//! - [`BseElectronShell`] - Individual shell data
//! - [`BseEcpPotential`] - ECP potential data
//! - [`BseRootMetadata`] - Basis set metadata
//! - [`BseGetBasisArgs`], [`BseGetBasisArgsBuilder`] - Arguments for
//!   `get_basis`
//! - [`BseFilterArgs`], [`BseFilterArgsBuilder`] - Arguments for
//!   `filter_basis_sets`
//!
//! ## Reader/Writer Functions
//! - [`read_formatted_basis_str`], [`read_formatted_basis_str_f`] - Parse basis
//!   set from string
//! - [`write_formatted_basis_str`], [`write_formatted_basis_str_f`] - Format
//!   basis set to string
//! - [`get_reader_formats`] - Get available input formats
//! - [`get_writer_formats`] - Get available output formats
//! - [`get_format_extension`] - Get file extension for a format
//!
//! # For Developers
//!
//! Internal utilities are also exported for use within the crate:
//! - Caching macros (`cached`, `once`)
//! - Serialization traits (`Serialize`, `Deserialize`)
//! - Collection types (`HashMap`, `HashSet`, `BTreeMap`)
//! - Error handling (`BseError`)

#![allow(unused_imports)]

// for users

pub use crate::api::*;
pub use crate::api::{
    filter_basis_sets, filter_basis_sets_f, get_all_basis_names, get_all_basis_names_f, get_basis_notes,
    get_basis_notes_f, get_families, get_families_f, get_family_notes, get_family_notes_f, get_reference_data,
    get_reference_data_f, get_reference_formats, get_references, get_references_f, get_references_formatted,
    get_references_formatted_f, get_roles, has_basis_notes, has_basis_notes_f, has_family_notes, has_family_notes_f,
    lookup_basis_by_role, lookup_basis_by_role_f, BseFilterArgs, BseFilterArgsBuilder,
};
pub use crate::dir_reader::{read_basis_from_dir, read_basis_from_dir_f};
pub use crate::dir_writer::{write_basis_to_dir, write_basis_to_dir_f};
pub use crate::fields::{
    BseAuxiliary, BseBasis, BseBasisElement, BseBasisMinimal, BseBasisReference, BseEcpElement, BseEcpPotential,
    BseElectronShell, BseElementComponents, BseElementReferences, BseGtoElement, BseMolssiBseSchema, BseReferenceEntry,
    BseReferenceInfoWithData, BseRootMetadata, BseRootMetadataVer, BseSkelComponentEcp, BseSkelComponentGto,
    BseSkelElement, BseSkelMetadata, BseSkelTable,
};
pub use crate::notes::process_notes;
pub use crate::readers::read::{get_reader_formats, read_formatted_basis_str, read_formatted_basis_str_f};
pub use crate::refconverters::{convert_references, get_reference_format_extension};
pub use crate::references::compact_references;
pub use crate::writers::write::{
    get_format_extension, get_writer_formats, is_dir_format, strip_dir_prefix, write_formatted_basis_str,
    write_formatted_basis_str_f,
};

#[cfg(feature = "remote")]
pub use crate::client::{
    get_api_url, get_basis_notes_remote, get_basis_remote, get_family_notes_remote, get_formats_remote,
    get_formatted_basis_remote, get_metadata_remote, get_reference_formats_remote, DEFAULT_API_URL,
};

// for developers

pub(crate) use cached::proc_macro::{cached, once};
pub(crate) use derive_builder::{Builder, UninitializedFieldError};
pub(crate) use duplicate::duplicate_item;
pub(crate) use itertools::*;
pub(crate) use regex::{Regex, RegexBuilder};
pub(crate) use serde::de::{Unexpected, Visitor};
pub(crate) use serde::{Deserialize, Deserializer, Serialize};
pub(crate) use std::collections::{BTreeMap, HashMap, HashSet};
pub(crate) use std::panic::catch_unwind;
pub(crate) use std::sync::Mutex;

pub(crate) use crate::error::BseError;
pub(crate) use crate::*;
pub(crate) use misc::{COMPACT, HIJ, HIK, INCOMPACT, SCIFMT_D, SCIFMT_E};
