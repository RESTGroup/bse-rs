//! Directory-based basis set writer.
//!
//! This module provides functionality to write basis sets in a directory
//! format, where each element is stored in a separate file within a directory.
//!
//! Directory structure: `<basis-directory>/<element-symbol>.<extension>`
//!
//! Supports all existing formats with the underlying format name (e.g., `json`,
//! `nwchem`). The `dir-` prefix handling is done at the CLI level.

use std::path::Path;

use crate::prelude::*;

/// Write a basis set to a directory with one file per element.
///
/// # Arguments
///
/// * `basis` - The basis set to write
/// * `dir_path` - Directory path where element files will be written
/// * `fmt` - Underlying format name (e.g., "json", "nwchem" - without "dir-"
///   prefix)
///
/// # Panics
///
/// Panics if the directory cannot be created or files cannot be written.
///
/// # Directory Structure
///
/// Files are written to `<dir_path>/<element-symbol>.<extension>` where:
/// - `<element-symbol>` is capitalized (e.g., "Fe", "H", "C")
/// - `<extension>` is determined by the format
///
/// # Example
///
/// ```rust,no_run
/// use bse::prelude::*;
/// use std::path::Path;
///
/// let basis = get_basis("def2-TZVP", BseGetBasisArgsBuilder::default().build().unwrap());
/// write_basis_to_dir(&basis, Path::new("/tmp/def2-tzvp"), "json");
/// // Creates: /tmp/def2-tzvp/H.json, C.json, etc.
/// ```
pub fn write_basis_to_dir(basis: &BseBasis, dir_path: &Path, fmt: &str) {
    write_basis_to_dir_f(basis, dir_path, fmt).unwrap()
}

pub fn write_basis_to_dir_f(basis: &BseBasis, dir_path: &Path, fmt: &str) -> Result<(), BseError> {
    let fmt_lower = fmt.to_lowercase();

    // Create the output directory
    std::fs::create_dir_all(dir_path)
        .map_err(|e| BseError::IOError(format!("Failed to create directory {}: {}", dir_path.display(), e)))?;

    // Get extension for the format
    let extension = get_format_extension(&fmt_lower)?;

    // Write each element to a separate file
    for (z_str, element_data) in &basis.elements {
        // Parse atomic number from string key
        let z: i32 =
            z_str.parse().map_err(|_| BseError::ValueError(format!("Invalid atomic number key: {}", z_str)))?;

        // Get capitalized element symbol for filename
        let element_sym = lut::element_sym_from_Z_with_normalize(z)
            .map_or(bse_raise!(ValueError, "Unknown atomic number: {}", z), Ok)?;

        // Construct filename
        let filename = format!("{}{}", element_sym, extension);
        let file_path = dir_path.join(&filename);

        // Write the element data
        write_element_to_file(element_data, &file_path, &fmt_lower, &basis.function_types)?;
    }

    Ok(())
}

/// Write a single element's basis data to a file.
///
/// For JSON format, writes the BseBasisElement directly.
/// For other formats, creates a single-element BseBasis and uses the format
/// writer.
fn write_element_to_file(
    element_data: &BseBasisElement,
    file_path: &Path,
    fmt: &str,
    function_types: &[String],
) -> Result<(), BseError> {
    let content = if fmt == "json" || fmt == "bsejson" {
        // For JSON format, write BseBasisElement directly
        serde_json::to_string_pretty(element_data)
            .map_err(|e| BseError::SerdeJsonError(format!("Failed to serialize element: {}", e)))?
    } else {
        // For other formats, create a single-element basis and use the writer
        let single_basis = create_single_element_basis(element_data, function_types);
        write_formatted_basis_str(&single_basis, fmt, None)
    };

    std::fs::write(file_path, content)
        .map_err(|e| BseError::IOError(format!("Failed to write file {}: {}", file_path.display(), e)))?;

    Ok(())
}

/// Create a BseBasis containing only a single element.
///
/// This is needed for non-JSON formats which expect a full BseBasis structure.
fn create_single_element_basis(element_data: &BseBasisElement, function_types: &[String]) -> BseBasis {
    // Use a placeholder key - the actual atomic number isn't needed for formatting
    // since writers use element symbols from the data
    let mut elements = HashMap::new();
    elements.insert("1".to_string(), element_data.clone());

    BseBasis {
        molssi_bse_schema: BseMolssiBseSchema {
            schema_type: "complete".to_string(),
            schema_version: "1.0".to_string(),
        },
        revision_description: String::new(),
        revision_date: String::new(),
        elements,
        version: String::new(),
        function_types: function_types.to_vec(),
        names: Vec::new(),
        tags: Vec::new(),
        family: String::new(),
        description: String::new(),
        role: "orbital".to_string(),
        auxiliaries: HashMap::new(),
        name: "element".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_basis_to_dir_json() {
        let args = BseGetBasisArgsBuilder::default().elements("H, C, O".to_string()).build().unwrap();
        let basis = get_basis("sto-3g", args);

        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("bse_test_write_json");
        let _ = std::fs::remove_dir_all(&test_dir);

        write_basis_to_dir(&basis, &test_dir, "json");

        // Check directory structure - files are written directly to test_dir
        assert!(test_dir.exists());
        assert!(test_dir.join("H.json").exists());
        assert!(test_dir.join("C.json").exists());
        assert!(test_dir.join("O.json").exists());

        // Read and verify content
        let h_content = std::fs::read_to_string(test_dir.join("H.json")).unwrap();
        let h_element: BseBasisElement = serde_json::from_str(&h_content).unwrap();
        assert!(h_element.electron_shells.is_some());

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_write_basis_to_dir_nwchem() {
        let args = BseGetBasisArgsBuilder::default().elements("H, C".to_string()).build().unwrap();
        let basis = get_basis("sto-3g", args);

        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("bse_test_write_nwchem");
        let _ = std::fs::remove_dir_all(&test_dir);

        write_basis_to_dir(&basis, &test_dir, "nwchem");

        // Check directory structure - files are written directly to test_dir
        assert!(test_dir.exists());
        assert!(test_dir.join("H.nw").exists());
        assert!(test_dir.join("C.nw").exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
