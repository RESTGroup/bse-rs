//! Directory-based basis set reader.
//!
//! This module provides functionality to read basis sets from a directory
//! format, where each element is stored in a separate file within a directory.
//!
//! Directory structure: `<basis-name>/<element-identifier>.<extension>`
//!
//! Element identifiers in filenames can be:
//! - Element symbol (e.g., "H", "Fe", "C") - case insensitive
//! - Element name (e.g., "Hydrogen", "Iron") - case insensitive
//! - Atomic number (e.g., "1", "26", "6")
//!
//! Supports all existing formats with the underlying format name (e.g., `json`,
//! `nwchem`). The `dir-` prefix handling is done at the CLI level.

use std::path::Path;

use crate::prelude::*;

/// Read a basis set from a directory with one file per element.
///
/// # Arguments
///
/// * `dir_path` - Path to the basis set directory (the directory containing
///   element files)
/// * `fmt` - Underlying format name (e.g., "json", "nwchem" - without "dir-"
///   prefix)
///
/// # Returns
///
/// A `BseBasisMinimal` containing all elements found in the directory.
///
/// # Panics
///
/// Panics if the directory cannot be read or files cannot be parsed.
///
/// # Filename Handling
///
/// Files must be named `<element-identifier>.<extension>` where:
/// - `<element-identifier>` can be element symbol, name, or atomic number (case
///   insensitive)
/// - `<extension>` is the format extension (e.g., `.json`, `.nw`)
///
/// Unrecognized files are skipped with a warning message.
/// If the same element appears multiple times (e.g., both "H.json" and
/// "1.json"), an error is raised.
///
/// # Example
///
/// ```rust,no_run
/// use bse::prelude::*;
/// use std::path::Path;
///
/// let basis = read_basis_from_dir(Path::new("/path/to/def2-tzvp"), "json");
/// println!("Found {} elements", basis.elements.len());
/// ```
pub fn read_basis_from_dir(dir_path: &Path, fmt: &str) -> BseBasisMinimal {
    read_basis_from_dir_f(dir_path, fmt).unwrap()
}

pub fn read_basis_from_dir_f(dir_path: &Path, fmt: &str) -> Result<BseBasisMinimal, BseError> {
    let fmt_lower = fmt.to_lowercase();

    // Verify directory exists
    if !dir_path.is_dir() {
        return bse_raise!(ValueError, "Path is not a directory: {}", dir_path.display());
    }

    // Get expected extension for the format
    let expected_ext = get_format_extension(&fmt_lower)?.trim_start_matches('.');

    // Get basis name from directory name
    let basis_name =
        dir_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string());

    // Track elements to detect clashes
    let mut elements: HashMap<String, BseBasisElement> = HashMap::new();
    let mut seen_z: HashMap<i32, String> = HashMap::new(); // z -> filename for clash detection
    let mut function_types: HashSet<String> = HashSet::new();

    // Read directory entries
    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| BseError::IOError(format!("Failed to read directory {}: {}", dir_path.display(), e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| BseError::IOError(format!("Failed to read directory entry: {}", e)))?;

        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check extension
        let file_ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
        if file_ext.as_deref() != Some(expected_ext) {
            continue;
        }

        // Get filename stem (element identifier)
        let filename = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();

        // Parse element identifier
        let z = match lut::element_Z_from_str(&stem) {
            Some(z) => z,
            None => {
                eprintln!("Warning: Skipping unrecognized file: {}", filename);
                continue;
            },
        };

        // Check for clashes
        if let Some(prev_filename) = seen_z.get(&z) {
            return bse_raise!(
                ValueError,
                "Element clash detected: '{}' and '{}' both refer to element {} (Z={})",
                prev_filename,
                filename,
                lut::element_sym_from_Z_with_normalize(z).unwrap_or_else(|| format!("Z={}", z)),
                z
            );
        }

        // Read file content
        let content = std::fs::read_to_string(&path)
            .map_err(|e| BseError::IOError(format!("Failed to read file {}: {}", path.display(), e)))?;

        // Parse element data
        let element_data = if fmt_lower == "json" || fmt_lower == "bsejson" {
            // For JSON format, parse BseBasisElement directly
            serde_json::from_str(&content)
                .map_err(|e| BseError::SerdeJsonError(format!("Failed to parse JSON in {}: {}", filename, e)))?
        } else {
            // For other formats, use the reader and extract element
            let minimal = read_formatted_basis_str_f(&content, fmt)?;

            // Extract the single element
            if minimal.elements.len() != 1 {
                return bse_raise!(
                    ValueError,
                    "Expected single element in file {}, found {}",
                    filename,
                    minimal.elements.len()
                );
            }

            minimal.elements.into_values().next().unwrap()
        };

        // Collect function types
        if let Some(ref shells) = element_data.electron_shells {
            for shell in shells {
                function_types.insert(shell.function_type.clone());
            }
        }
        if let Some(ref potentials) = element_data.ecp_potentials {
            for pot in potentials {
                function_types.insert(pot.ecp_type.clone());
            }
        }

        // Store element
        let z_str = z.to_string();
        elements.insert(z_str, element_data);
        seen_z.insert(z, filename);
    }

    // Check that we found at least one element
    if elements.is_empty() {
        return bse_raise!(ValueError, "No valid element files found in directory: {}", dir_path.display());
    }

    Ok(BseBasisMinimal {
        molssi_bse_schema: BseMolssiBseSchema {
            schema_type: "complete".to_string(),
            schema_version: "1.0".to_string(),
        },
        elements,
        function_types: function_types.into_iter().collect(),
        name: basis_name,
        description: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_basis_from_dir_json() {
        // Read from the example directory
        let dir_path = Path::new("/home/a/rest_pack/rest/basis-set-pool/def2-tzvp");
        if !dir_path.exists() {
            eprintln!("Skipping test: example directory not found");
            return;
        }

        let basis = read_basis_from_dir(dir_path, "json");

        // Should have multiple elements
        assert!(!basis.elements.is_empty());

        // Check that common elements exist
        assert!(basis.elements.contains_key("1")); // H
        assert!(basis.elements.contains_key("6")); // C
        assert!(basis.elements.contains_key("26")); // Fe

        println!("Read {} elements from def2-tzvp", basis.elements.len());
    }

    #[test]
    fn test_read_with_mixed_naming() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("bse_test_mixed_naming");
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create files with different naming styles
        let h_file = test_dir.join("H.json");
        let c_file = test_dir.join("6.json"); // Using atomic number

        let h_element = BseBasisElement {
            references: Vec::new(),
            electron_shells: Some(vec![BseElectronShell {
                function_type: "gto".to_string(),
                region: String::new(),
                angular_momentum: vec![0],
                exponents: vec!["1.0".to_string()],
                coefficients: vec![vec!["1.0".to_string()]],
            }]),
            ecp_potentials: None,
            ecp_electrons: None,
        };

        let c_element = BseBasisElement {
            references: Vec::new(),
            electron_shells: Some(vec![BseElectronShell {
                function_type: "gto".to_string(),
                region: String::new(),
                angular_momentum: vec![0, 1],
                exponents: vec!["1.0".to_string()],
                coefficients: vec![vec!["1.0".to_string()], vec!["1.0".to_string()]],
            }]),
            ecp_potentials: None,
            ecp_electrons: None,
        };

        std::fs::write(&h_file, serde_json::to_string_pretty(&h_element).unwrap()).unwrap();
        std::fs::write(&c_file, serde_json::to_string_pretty(&c_element).unwrap()).unwrap();

        let basis = read_basis_from_dir(&test_dir, "json");

        assert_eq!(basis.elements.len(), 2);
        assert!(basis.elements.contains_key("1")); // H
        assert!(basis.elements.contains_key("6")); // C

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_clash_detection() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("bse_test_clash");
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create two files for the same element (H = 1)
        let h1_file = test_dir.join("H.json");
        let h2_file = test_dir.join("1.json");

        let element = BseBasisElement {
            references: Vec::new(),
            electron_shells: None,
            ecp_potentials: None,
            ecp_electrons: None,
        };

        std::fs::write(&h1_file, serde_json::to_string_pretty(&element).unwrap()).unwrap();
        std::fs::write(&h2_file, serde_json::to_string_pretty(&element).unwrap()).unwrap();

        let result = read_basis_from_dir_f(&test_dir, "json");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("clash"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
