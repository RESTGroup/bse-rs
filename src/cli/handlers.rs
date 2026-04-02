//! Handlers for CLI subcommands.
//!
//! This module contains the implementation of each CLI subcommand,
//! following the pattern of Python's `bse_handlers.py`.

use std::path::PathBuf;

use crate::prelude::*;
use crate::{bse_raise, misc::compact_elements, BseDataSource, BseError};

use super::check::{detect_dir_format_from_files, detect_format_from_extension};
use super::common::{format_columns, format_map_columns, format_table, get_cli_only_formats, resolve_cli_format};

/// Handle the `list-writer-formats` subcommand.
pub fn handle_list_writer_formats(no_description: bool) -> Result<String, BseError> {
    let formats = get_writer_formats_with_aliases(None);

    if no_description {
        let names: Vec<String> = formats
            .iter()
            .flat_map(|(name, (_, aliases))| std::iter::once(name.clone()).chain(aliases.clone()))
            .chain(get_cli_only_formats().iter().map(|(name, _, _)| name.clone()))
            .collect();
        // Sort alphabetically
        let mut sorted_names = names;
        sorted_names.sort();
        Ok(sorted_names.join("\n"))
    } else {
        // Format as table: name | aliases | display
        let headers = ["Name", "Aliases", "Display"];
        let mut rows: Vec<Vec<String>> = formats
            .iter()
            .map(|(name, (display, aliases))| vec![name.clone(), aliases.join(", "), display.clone()])
            .collect();
        // Add CLI-only formats
        for (name, aliases, display) in get_cli_only_formats() {
            rows.push(vec![name, aliases, display]);
        }
        // Sort alphabetically by name
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        Ok(format_table(&headers, &rows).join("\n"))
    }
}

/// Handle the `list-reader-formats` subcommand.
pub fn handle_list_reader_formats(no_description: bool) -> Result<String, BseError> {
    let formats = get_reader_formats_with_aliases();

    if no_description {
        let names: Vec<String> = formats
            .iter()
            .flat_map(|(name, (_, aliases))| std::iter::once(name.clone()).chain(aliases.clone()))
            .chain(get_cli_only_formats().iter().map(|(name, _, _)| name.clone()))
            .collect();
        // Sort alphabetically
        let mut sorted_names = names;
        sorted_names.sort();
        Ok(sorted_names.join("\n"))
    } else {
        // Format as table: name | aliases | display
        let headers = ["Name", "Aliases", "Display"];
        let mut rows: Vec<Vec<String>> = formats
            .iter()
            .map(|(name, (display, aliases))| vec![name.clone(), aliases.join(", "), display.clone()])
            .collect();
        // Add CLI-only formats
        for (name, aliases, display) in get_cli_only_formats() {
            rows.push(vec![name, aliases, display]);
        }
        // Sort alphabetically by name
        rows.sort_by(|a, b| a[0].cmp(&b[0]));
        Ok(format_table(&headers, &rows).join("\n"))
    }
}

/// Handle the `list-ref-formats` subcommand.
pub fn handle_list_ref_formats(no_description: bool) -> Result<String, BseError> {
    let formats = get_reference_formats();

    if no_description {
        Ok(formats.keys().cloned().collect::<Vec<_>>().join("\n").to_string())
    } else {
        let items: Vec<(String, String)> = formats.into_iter().collect();
        Ok(format_map_columns(&items, "").join("\n"))
    }
}

/// Handle the `list-roles` subcommand.
pub fn handle_list_roles(no_description: bool) -> Result<String, BseError> {
    let roles = get_roles();

    if no_description {
        Ok(roles.keys().cloned().collect::<Vec<_>>().join("\n").to_string())
    } else {
        let items: Vec<(&str, &str)> = roles.into_iter().collect();
        Ok(format_map_columns(&items, "").join("\n"))
    }
}

/// Handle the `get-data-dir` subcommand.
pub fn handle_get_data_dir() -> Result<String, BseError> {
    match get_bse_data_dir() {
        Some(dir) => Ok(dir),
        None => bse_raise!(ValueError, "No data directory available. Set BSE_DATA_DIR environment variable."),
    }
}

/// Handle the `list-basis-sets` subcommand.
///
/// Lists basis sets with optional filtering by family, role, substring, or
/// elements. Output is grouped by family and sorted alphabetically within
/// each family. Basis sets without a family are shown at the end.
pub fn handle_list_basis_sets(
    substr: Option<String>,
    family: Option<String>,
    role: Option<String>,
    elements: Option<String>,
    data_dir: Option<String>,
    no_description: bool,
) -> Result<String, BseError> {
    let args = BseFilterArgsBuilder::default()
        .substr(substr)
        .family(family)
        .role(role)
        .elements(elements)
        .data_dir(data_dir)
        .build()?;

    let metadata = filter_basis_sets(args);

    // Group by family, separating empty families
    let mut by_family: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    let mut no_family: Vec<(String, String)> = Vec::new();

    for v in metadata.values() {
        if v.family.is_empty() {
            no_family.push((v.display_name.clone(), v.description.clone()));
        } else {
            by_family.entry(v.family.clone()).or_default().push((v.display_name.clone(), v.description.clone()));
        }
    }

    // Sort within each family
    for items in by_family.values_mut() {
        items.sort_by(|a, b| a.0.cmp(&b.0));
    }
    no_family.sort_by(|a, b| a.0.cmp(&b.0));

    if no_description {
        // Simple list: just names, sorted by family then alphabetically
        let mut result = Vec::new();
        for (_family, items) in &by_family {
            for (name, _) in items {
                result.push(name.clone());
            }
        }
        for (name, _) in &no_family {
            result.push(name.clone());
        }
        Ok(result.join("\n"))
    } else {
        // Format as multi-table with borders
        let mut result = Vec::new();

        // Process families in order (BTreeMap is sorted)
        for (family, items) in by_family {
            result.push(format!("\n[{}]", family));
            let headers = ["Name", "Description"];
            let rows: Vec<Vec<String>> = items.into_iter().map(|(n, d)| vec![n, d]).collect();
            result.extend(format_table(&headers, &rows));
        }

        // Add uncategorized at the end if any
        if !no_family.is_empty() {
            result.push("\n[uncategorized]".to_string());
            let headers = ["Name", "Description"];
            let rows: Vec<Vec<String>> = no_family.into_iter().map(|(n, d)| vec![n, d]).collect();
            result.extend(format_table(&headers, &rows));
        }

        // Remove leading newline if present
        if !result.is_empty() && result[0].starts_with('\n') {
            result[0] = result[0][1..].to_string();
        }
        Ok(result.join("\n"))
    }
}

/// Handle the `list-families` subcommand.
pub fn handle_list_families(data_dir: Option<String>) -> Result<String, BseError> {
    let families = get_families(data_dir);
    Ok(families.join("\n"))
}

/// Handle the `lookup-by-role` subcommand.
pub fn handle_lookup_by_role(basis: String, role: String, data_dir: Option<String>) -> Result<String, BseError> {
    let aux_names = lookup_basis_by_role(&basis, &role, data_dir);
    Ok(aux_names.join("\n"))
}

/// Handle the `get-basis` subcommand.
///
/// Outputs a formatted basis set with all optional manipulations.
/// Supports both single-file and directory formats.
#[allow(clippy::too_many_arguments)]
pub fn handle_get_basis(
    basis: String,
    fmt: String,
    elements: Option<String>,
    version: Option<String>,
    noheader: bool,
    unc_gen: bool,
    unc_spdf: bool,
    unc_seg: bool,
    rm_free: bool,
    opt_gen: bool,
    make_gen: bool,
    aug_diffuse: i32,
    aug_steep: i32,
    get_aux: i32,
    data_dir: Option<String>,
    output_path: Option<PathBuf>,
    source: BseDataSource,
) -> Result<String, BseError> {
    let args = BseGetBasisArgsBuilder::default()
        .elements(elements)
        .version(version)
        .header(!noheader)
        .uncontract_general(unc_gen)
        .uncontract_spdf(unc_spdf)
        .uncontract_segmented(unc_seg)
        .remove_free_primitives(rm_free)
        .optimize_general(opt_gen)
        .make_general(make_gen)
        .augment_diffuse(aug_diffuse)
        .augment_steep(aug_steep)
        .get_aux(get_aux)
        .data_dir(data_dir)
        .source(source)
        .build()?;

    // Resolve CLI-only format aliases (e.g., "rest" -> "dir-json")
    let resolved_fmt = resolve_cli_format(&fmt);

    // Check if directory format
    if is_dir_format(&resolved_fmt) {
        // For directory format, output_path is required
        let dir_path = output_path.unwrap_or(basis.as_str().into());

        let underlying_fmt = strip_dir_prefix(&resolved_fmt);
        let basis_data = get_basis(&basis, args);
        write_basis_to_dir_f(&basis_data, &dir_path, underlying_fmt)?;

        Ok(format!("Basis set '{}' written to {}", basis, dir_path.display()))
    } else {
        Ok(get_formatted_basis(&basis, &resolved_fmt, args))
    }
}

/// Handle the `get-refs` subcommand.
pub fn handle_get_refs(
    basis: String,
    reffmt: String,
    elements: Option<String>,
    version: Option<String>,
    _data_dir: Option<String>,
) -> Result<String, BseError> {
    Ok(get_references_formatted(&basis, elements.as_deref(), version.as_deref(), &reffmt))
}

/// Handle the `get-info` subcommand.
///
/// Outputs detailed metadata about a basis set.
pub fn handle_get_info(basis: String, data_dir: Option<String>) -> Result<String, BseError> {
    let resolved_data_dir = data_dir.clone().or(get_bse_data_dir());
    if resolved_data_dir.is_none() {
        return bse_raise!(ValueError, "No data directory available. Set BSE_DATA_DIR environment variable.");
    }

    let metadata = get_metadata(&resolved_data_dir.unwrap());
    let tr_name = crate::misc::transform_basis_name(&basis);

    if !metadata.contains_key(&tr_name) {
        return bse_raise!(ValueError, "Basis set '{}' does not exist.", basis);
    }

    let bs_meta = &metadata[&tr_name];
    let mut ret = Vec::new();

    ret.push("-".repeat(80));
    ret.push(basis.clone());
    ret.push("-".repeat(80));
    ret.push(format!("    Display Name: {}", bs_meta.display_name));
    ret.push(format!("     Description: {}", bs_meta.description));
    ret.push(format!("            Role: {}", bs_meta.role));
    ret.push(format!("          Family: {}", bs_meta.family));
    ret.push(format!("  Function Types: {}", bs_meta.function_types.join(",")));
    ret.push(format!("  Latest Version: {}", bs_meta.latest_version));
    ret.push(String::new());

    // Auxiliary basis sets
    if bs_meta.auxiliaries.is_empty() {
        ret.push("Auxiliary Basis Sets: None".to_string());
    } else {
        ret.push("Auxiliary Basis Sets:".to_string());
        let aux_items: Vec<(String, String)> = bs_meta
            .auxiliaries
            .iter()
            .map(|(k, v)| match v {
                BseAuxiliary::Str(s) => (k.clone(), s.clone()),
                BseAuxiliary::Vec(v) => (k.clone(), v.join(", ")),
            })
            .collect();
        ret.extend(format_map_columns(&aux_items, "    "));
    }

    // Versions
    ret.push(String::new());
    ret.push("Versions:".to_string());
    let version_items: Vec<Vec<String>> = bs_meta
        .versions
        .iter()
        .map(|(k, v)| {
            vec![
                k.clone(),
                v.revdate.clone(),
                compact_elements(&v.elements.iter().filter_map(|e| e.parse::<i32>().ok()).collect::<Vec<_>>()),
                v.revdesc.clone(),
            ]
        })
        .collect();
    ret.extend(format_columns(&version_items, "    "));

    Ok(ret.join("\n"))
}

/// Handle the `get-notes` subcommand.
pub fn handle_get_notes(basis: String, data_dir: Option<String>) -> Result<String, BseError> {
    Ok(get_basis_notes(&basis, data_dir))
}

/// Handle the `get-family` subcommand.
pub fn handle_get_family(basis: String, data_dir: Option<String>) -> Result<String, BseError> {
    let resolved_data_dir = data_dir.clone().or(get_bse_data_dir());
    if resolved_data_dir.is_none() {
        return bse_raise!(ValueError, "No data directory available. Set BSE_DATA_DIR environment variable.");
    }

    let metadata = get_metadata(&resolved_data_dir.unwrap());
    let tr_name = crate::misc::transform_basis_name(&basis);

    if !metadata.contains_key(&tr_name) {
        return bse_raise!(ValueError, "Basis set '{}' does not exist.", basis);
    }

    Ok(metadata[&tr_name].family.clone())
}

/// Handle the `get-versions` subcommand.
pub fn handle_get_versions(basis: String, data_dir: Option<String>, no_description: bool) -> Result<String, BseError> {
    let resolved_data_dir = data_dir.clone().or(get_bse_data_dir());
    if resolved_data_dir.is_none() {
        return bse_raise!(ValueError, "No data directory available. Set BSE_DATA_DIR environment variable.");
    }

    let metadata = get_metadata(&resolved_data_dir.unwrap());
    let tr_name = crate::misc::transform_basis_name(&basis);

    if !metadata.contains_key(&tr_name) {
        return bse_raise!(ValueError, "Basis set '{}' does not exist.", basis);
    }

    let versions = &metadata[&tr_name].versions;

    if no_description {
        Ok(versions.keys().cloned().collect::<Vec<_>>().join("\n"))
    } else {
        let items: Vec<(String, String)> = versions.iter().map(|(k, v)| (k.clone(), v.revdesc.clone())).collect();
        Ok(format_map_columns(&items, "").join("\n"))
    }
}

/// Handle the `get-family-notes` subcommand.
pub fn handle_get_family_notes(family: String, data_dir: Option<String>) -> Result<String, BseError> {
    Ok(get_family_notes(&family, data_dir))
}

/// Handle the `convert-basis` subcommand.
///
/// Converts a basis set file from one format to another.
/// Supports both single-file and directory formats.
pub fn handle_convert_basis(
    input_file: PathBuf,
    output_file: PathBuf,
    in_fmt: Option<String>,
    out_fmt: Option<String>,
    make_gen: bool,
) -> Result<String, BseError> {
    // Check if paths exist and are directories
    let input_is_dir = input_file.is_dir();
    let output_exists = output_file.exists();
    let output_is_dir = output_file.is_dir();

    // Detect input format
    let resolved_in_fmt = in_fmt.map(|f| resolve_cli_format(&f)).or_else(|| {
        if input_is_dir {
            detect_dir_format_from_files(&input_file, true)
        } else {
            detect_format_from_extension(&input_file.to_string_lossy(), true)
        }
    });

    if resolved_in_fmt.is_none() {
        return bse_raise!(
            ValueError,
            "Could not detect input format from '{}'. Specify format with --in-fmt",
            input_file.display()
        );
    }

    // Detect output format with strict validation
    // User-specified format takes precedence
    let resolved_out_fmt = if let Some(fmt) = &out_fmt {
        let resolved = resolve_cli_format(fmt);
        let is_dir_fmt = is_dir_format(&resolved);

        // Validate: dir format can't be written to existing file
        if output_exists && !output_is_dir && is_dir_fmt {
            return bse_raise!(
                ValueError,
                "Cannot write directory format '{}' to a file path '{}'. Use a directory path instead.",
                fmt,
                output_file.display()
            );
        }

        // Validate: non-dir format can't be written to existing directory
        if output_exists && output_is_dir && !is_dir_fmt {
            return bse_raise!(
                ValueError,
                "Cannot write single-file format '{}' to a directory '{}'. Use a file path instead.",
                fmt,
                output_file.display()
            );
        }

        Some(resolved)
    } else {
        // No --out-fmt specified - try auto-detection
        if output_is_dir {
            // Existing directory: try to detect from files inside
            detect_dir_format_from_files(&output_file, false)
        } else if output_exists {
            // Existing file: detect from extension
            detect_format_from_extension(&output_file.to_string_lossy(), false)
        } else {
            // Output doesn't exist: detect from extension in filename
            detect_format_from_extension(&output_file.to_string_lossy(), false)
        }
    };

    if resolved_out_fmt.is_none() {
        if output_is_dir {
            return bse_raise!(
                ValueError,
                "Could not detect output format from directory '{}'. Specify format with --out-fmt",
                output_file.display()
            );
        } else {
            return bse_raise!(
                ValueError,
                "Could not detect output format from filename '{}'. Specify format with --out-fmt",
                output_file.display()
            );
        }
    }

    let in_fmt_resolved = resolved_in_fmt.unwrap();
    let out_fmt_resolved = resolved_out_fmt.unwrap();

    // Handle directory formats
    let input_dir_mode = is_dir_format(&in_fmt_resolved) || input_is_dir;
    let output_dir_mode = is_dir_format(&out_fmt_resolved) || output_is_dir;

    if input_dir_mode && output_dir_mode {
        // Directory to directory conversion
        let underlying_in_fmt = strip_dir_prefix(&in_fmt_resolved);
        let underlying_out_fmt = strip_dir_prefix(&out_fmt_resolved);

        let mut basis = read_basis_from_dir(&input_file, underlying_in_fmt);

        // Apply make_general if requested
        if make_gen {
            let mut full_basis = BseBasis::from_minimal(basis);
            crate::manip::make_general(&mut full_basis, false);
            crate::manip::prune_basis(&mut full_basis);
            basis = BseBasisMinimal {
                molssi_bse_schema: full_basis.molssi_bse_schema,
                elements: full_basis.elements,
                function_types: full_basis.function_types,
                name: full_basis.name,
                description: full_basis.description,
            };
        }

        write_basis_to_dir_f(&BseBasis::from_minimal(basis), &output_file, underlying_out_fmt)?;

        return Ok(format!("Converted {} -> {}", input_file.display(), output_file.display()));
    }

    if input_dir_mode {
        // Directory to file conversion
        let underlying_in_fmt = strip_dir_prefix(&in_fmt_resolved);
        let mut basis = read_basis_from_dir(&input_file, underlying_in_fmt);

        if make_gen {
            let mut full_basis = BseBasis::from_minimal(basis);
            crate::manip::make_general(&mut full_basis, false);
            crate::manip::prune_basis(&mut full_basis);
            basis = BseBasisMinimal {
                molssi_bse_schema: full_basis.molssi_bse_schema,
                elements: full_basis.elements,
                function_types: full_basis.function_types,
                name: full_basis.name,
                description: full_basis.description,
            };
        }

        let full_basis = BseBasis::from_minimal(basis);
        let output_str = write_formatted_basis_str(&full_basis, &out_fmt_resolved, None);
        std::fs::write(&output_file, output_str)?;

        return Ok(format!("Converted {} -> {}", input_file.display(), output_file.display()));
    }

    if output_dir_mode {
        // File to directory conversion
        let underlying_out_fmt = strip_dir_prefix(&out_fmt_resolved);

        let input_str = std::fs::read_to_string(&input_file)?;
        let basis_minimal = read_formatted_basis_str(&input_str, &in_fmt_resolved);

        let mut basis = BseBasis::from_minimal(basis_minimal);

        if make_gen {
            crate::manip::make_general(&mut basis, false);
            crate::manip::prune_basis(&mut basis);
        }

        write_basis_to_dir_f(&basis, &output_file, underlying_out_fmt)?;

        return Ok(format!("Converted {} -> {}", input_file.display(), output_file.display()));
    }

    // Standard file to file conversion
    let input_str = std::fs::read_to_string(&input_file)?;
    let basis_minimal = read_formatted_basis_str(&input_str, &in_fmt_resolved);

    // Convert to full BseBasis for manipulation
    let mut basis = BseBasis::from_minimal(basis_minimal);

    // Apply make_general if requested
    if make_gen {
        crate::manip::make_general(&mut basis, false);
        crate::manip::prune_basis(&mut basis);
    }

    // Write the output
    let output_str = write_formatted_basis_str(&basis, &out_fmt_resolved, None);
    std::fs::write(&output_file, output_str)?;

    Ok(format!("Converted {} -> {}", input_file.display(), output_file.display()))
}

/// Handle the `autoaux-basis` subcommand.
///
/// Generates an AutoAux auxiliary basis from an orbital basis file.
pub fn handle_autoaux_basis(
    input_file: PathBuf,
    output_file: PathBuf,
    in_fmt: Option<String>,
    out_fmt: Option<String>,
) -> Result<String, BseError> {
    // Detect formats from file extensions if not specified
    let resolved_in_fmt = in_fmt.or_else(|| detect_format_from_extension(&input_file.to_string_lossy(), true));
    let resolved_out_fmt = out_fmt.or_else(|| detect_format_from_extension(&output_file.to_string_lossy(), false));

    if resolved_in_fmt.is_none() {
        return bse_raise!(
            ValueError,
            "Could not detect input format from filename '{}'. Specify format with --in-fmt",
            input_file.display()
        );
    }
    if resolved_out_fmt.is_none() {
        return bse_raise!(
            ValueError,
            "Could not detect output format from filename '{}'. Specify format with --out-fmt",
            output_file.display()
        );
    }

    // Read the input file
    let input_str = std::fs::read_to_string(&input_file)?;
    let basis_minimal = read_formatted_basis_str(&input_str, &resolved_in_fmt.unwrap());

    // Convert to full BseBasis
    let basis = BseBasis::from_minimal(basis_minimal);

    // Generate AutoAux basis
    let autoaux = crate::manip::autoaux_basis(&basis);

    // Write the output
    let output_str = write_formatted_basis_str(&autoaux, &resolved_out_fmt.unwrap(), None);
    std::fs::write(&output_file, output_str)?;

    Ok(format!("Orbital basis {} -> AutoAux basis {}", input_file.display(), output_file.display()))
}

/// Handle the `autoabs-basis` subcommand.
///
/// Generates an AutoABS auxiliary basis from an orbital basis file.
pub fn handle_autoabs_basis(
    input_file: PathBuf,
    output_file: PathBuf,
    in_fmt: Option<String>,
    out_fmt: Option<String>,
) -> Result<String, BseError> {
    // Detect formats from file extensions if not specified
    let resolved_in_fmt = in_fmt.or_else(|| detect_format_from_extension(&input_file.to_string_lossy(), true));
    let resolved_out_fmt = out_fmt.or_else(|| detect_format_from_extension(&output_file.to_string_lossy(), false));

    if resolved_in_fmt.is_none() {
        return bse_raise!(
            ValueError,
            "Could not detect input format from filename '{}'. Specify format with --in-fmt",
            input_file.display()
        );
    }
    if resolved_out_fmt.is_none() {
        return bse_raise!(
            ValueError,
            "Could not detect output format from filename '{}'. Specify format with --out-fmt",
            output_file.display()
        );
    }

    // Read the input file
    let input_str = std::fs::read_to_string(&input_file)?;
    let basis_minimal = read_formatted_basis_str(&input_str, &resolved_in_fmt.unwrap());

    // Convert to full BseBasis
    let basis = BseBasis::from_minimal(basis_minimal);

    // Generate AutoABS basis
    let autoabs = crate::manip::autoabs_basis(&basis, 1, 1.5);

    // Write the output
    let output_str = write_formatted_basis_str(&autoabs, &resolved_out_fmt.unwrap(), None);
    std::fs::write(&output_file, output_str)?;

    Ok(format!("Orbital basis {} -> AutoABS basis {}", input_file.display(), output_file.display()))
}
