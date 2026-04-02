//! Conversion of basis sets to Molcas basis_library format.

use crate::prelude::*;
use deunicode::deunicode;

/// Extract the first author from a reference key.
fn first_author(ref_key: Option<&str>, ref_data: &HashMap<String, BseReferenceEntry>) -> String {
    if ref_key.is_none() {
        return String::new();
    }
    let ref_key = ref_key.unwrap();

    if ref_key == "gaussian09e01" {
        return "Gaussian".to_string();
    }

    if let Some(entry) = ref_data.get(ref_key) {
        if let Some(author) = entry.get_field("authors").first() {
            // Take only the first author (before comma)
            let author = author.split(',').next().unwrap_or(author);
            // Convert to ASCII and replace spaces with underscores
            let author = deunicode(author);
            return author.replace(' ', "_");
        }
    }

    String::new()
}

/// Format an author name for citation.
///
/// Converts "Last, First" or "Last, First-Second" to "F. Last" format.
fn format_author(author: &str) -> String {
    let parts: Vec<&str> = author.split(',').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return String::new();
    }

    let last_name = parts[0];

    // Handle first name part
    let first_name_parts: Vec<&str> = if parts.len() > 1 { parts[1].split_whitespace().collect() } else { Vec::new() };

    // Python logic: if only one first name part, try hyphen splitting
    // Otherwise, no separator between parts (just concatenate initials)
    let (sep, first_name_parts): (String, Vec<&str>) = if first_name_parts.len() == 1 {
        if first_name_parts[0].contains('-') {
            (String::new(), first_name_parts[0].split('-').collect())
        } else {
            ("-".to_string(), first_name_parts)
        }
    } else {
        (String::new(), first_name_parts)
    };

    let mut text = String::new();
    for (i, n) in first_name_parts.iter().enumerate() {
        if i > 0 {
            text.push_str(&sep);
        }
        if n.ends_with('.') {
            text.push_str(n);
        } else if !n.is_empty() {
            text.push_str(&format!("{}.", n.chars().next().unwrap()));
        }
    }

    if !text.is_empty() {
        text.push(' ');
    }
    text.push_str(last_name);
    text
}

/// Format a reference for citation display.
fn format_reference(ref_key: Option<&str>, ref_data: &HashMap<String, BseReferenceEntry>) -> String {
    if ref_key.is_none() {
        return "Unknown reference".to_string();
    }
    let ref_key = ref_key.unwrap();

    if let Some(entry) = ref_data.get(ref_key) {
        let mut text = String::new();

        // Format authors
        let authors = entry.get_field("authors");
        if authors.len() > 9 {
            text.push_str(&format_author(&authors[0]));
            text.push_str(", et al.");
        } else {
            let formatted_authors: Vec<String> = authors.iter().map(|a| format_author(a)).collect();
            text.push_str(&formatted_authors.join(", "));
        }

        if !text.ends_with('.') {
            text.push('.');
        }

        // Add journal or book information
        if let Some(journal) = entry.get_field_opt("journal") {
            text.push_str(&format!(" {}", journal));
            if let Some(volume) = entry.get_field_opt("volume") {
                text.push_str(&format!(" {}", volume));
            }
            if let Some(year) = entry.get_field_opt("year") {
                text.push_str(&format!(" ({})", year));
            }
            if let Some(pages) = entry.get_field_opt("pages") {
                text.push_str(&format!(" {}", pages));
            }
        } else if let Some(booktitle) = entry.get_field_opt("booktitle") {
            text.push_str(&format!(" In \"{}\"", booktitle));
            if let Some(year) = entry.get_field_opt("year") {
                text.push_str(&format!(" ({})", year));
            }
            if let Some(pages) = entry.get_field_opt("pages") {
                text.push_str(&format!(" {}", pages));
            }
        } else if let Some(title) = entry.get_field_opt("title") {
            text.push_str(&format!(" {}", title));
        }

        if !text.ends_with('.') {
            text.push('.');
        }

        // Add DOI
        if let Some(doi) = entry.get_field_opt("doi") {
            text.push_str(&format!(" doi:{}", doi.to_lowercase()));
        }

        return deunicode(&text);
    }

    "Unknown reference".to_string()
}

/// Converts a basis set to Molcas basis_library format.
pub fn write_molcas_library(basis: &BseBasis) -> String {
    let mut basis = basis.clone();
    manip::make_general(&mut basis, false);
    sort::sort_basis(&mut basis);

    let mut s: Vec<String> = vec![];

    // Get reference data for formatting citations
    let ref_data = api::get_reference_data(None);

    for (z, data) in basis.elements.iter().sorted_by_key(|(z, _)| z.parse::<i32>().unwrap_or(0)) {
        let has_electron = data.electron_shells.is_some();
        let has_ecp = data.ecp_potentials.is_some();

        let el_name = lut::element_name_from_Z(z.parse().unwrap()).unwrap().to_uppercase();
        let el_sym = lut::element_sym_from_Z_with_normalize(z.parse().unwrap()).unwrap();

        // Get basis name
        let bs_name =
            if !basis.names.is_empty() { basis.names[0].replace(' ', "_") } else { basis.name.replace(' ', "_") };

        // Get contraction string
        let cont = if has_electron {
            data.electron_shells.as_ref().map_or("".to_string(), |shls| misc::contraction_string(shls, HIK, COMPACT))
        } else {
            "".to_string()
        };

        // Get reference key from the element's references (use the LAST reference and
        // LAST key)
        let ref_key =
            data.references.iter().next_back().and_then(|r| r.reference_keys.iter().next_back()).map(|s| s.as_str());

        let author = first_author(ref_key, &ref_data);

        // Number of electrons
        let mut nelectrons = z.parse::<i32>().unwrap();
        let mut ecp_str = String::new();
        if has_ecp {
            nelectrons -= data.ecp_electrons.unwrap();
            ecp_str = format!("ECP.{}el.", nelectrons);
        }

        // Header line: /SYMBOL.BASIS_NAME.AUTHOR.CONTRACTION.ECP/
        s.push(format!("/{el_sym}.{bs_name}.{author}.{cont}.{ecp_str}"));

        // Reference citation
        s.push(format_reference(ref_key, &ref_data));

        // Element name and contraction
        let cont_full =
            data.electron_shells.as_ref().map_or("".to_string(), |shls| misc::contraction_string(shls, HIK, INCOMPACT));
        s.push(format!("{} {}", el_name, cont_full));

        if has_electron {
            // Are there cartesian shells?
            let mut cartesian_shells = HashSet::new();
            for shell in data.electron_shells.as_ref().unwrap() {
                if shell.function_type == "gto_cartesian" {
                    for am in &shell.angular_momentum {
                        cartesian_shells.insert(lut::amint_to_char(&[*am], HIK));
                    }
                }
            }
            if !cartesian_shells.is_empty() {
                s.push("Options".to_string());
                s.push(format!("Cartesian {}", cartesian_shells.iter().join(" ")));
                s.push("EndOptions".to_string());
            }

            let shells = data.electron_shells.as_ref().unwrap();
            let max_am = misc::max_am(shells);

            s.push(format!("{nelectrons:>7}.0   {max_am}"));

            for shell in shells {
                let exponents = &shell.exponents;
                let coefficients = &shell.coefficients;
                let nprim = exponents.len();
                let ngen = coefficients.len();

                let amchar = lut::amint_to_char(&shell.angular_momentum, HIK).to_lowercase();
                s.push(format!("* {amchar}-type functions"));
                s.push(format!("{nprim:>6}    {ngen}"));

                s.push(printing::write_matrix(std::slice::from_ref(exponents), &[17], SCIFMT_E));

                let point_places = (1..=ngen).map(|i| 8 * i + 15 * (i - 1)).collect_vec();
                s.push(printing::write_matrix(coefficients, &point_places, SCIFMT_E));
            }
        }

        if has_ecp {
            let ecp_potentials = data.ecp_potentials.as_ref().unwrap();
            let max_ecp_am = ecp_potentials.iter().map(|x| x.angular_momentum[0]).max().unwrap();

            // Sort lowest->highest, then put the highest at the beginning
            let mut ecp_list =
                ecp_potentials.iter().sorted_by(|a, b| a.angular_momentum.cmp(&b.angular_momentum)).collect_vec();
            let ecp_list_last = ecp_list.pop().unwrap();
            ecp_list.insert(0, ecp_list_last);

            s.push(format!("PP, {}, {}, {} ;", el_sym, data.ecp_electrons.unwrap(), max_ecp_am));

            for pot in ecp_list {
                let rexponents = &pot.r_exponents;
                let gexponents = &pot.gaussian_exponents;
                let coefficients = &pot.coefficients;

                let am = &pot.angular_momentum;
                let amchar = lut::amint_to_char(am, HIK);

                if am[0] == max_ecp_am {
                    s.push(format!("{}; !  ul potential", rexponents.len()));
                } else {
                    s.push(format!("{}; !  {amchar}-ul potential", rexponents.len()));
                }

                for p in 0..rexponents.len() {
                    s.push(format!("{},{},{};", rexponents[p], gexponents[p], coefficients[0][p]));
                }
            }

            s.push("Spectral Representation Operator".to_string());
            s.push("End of Spectral Representation Operator".to_string());
        }

        s.push("".to_string()); // extra newline
    }

    s.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_molcas_library() {
        let args = BseGetBasisArgsBuilder::default().elements("1, 49".to_string()).build().unwrap();
        let basis = get_basis("def2-TZVP", args);
        let output = write_molcas_library(&basis);
        println!("{output}");
    }

    #[test]
    fn test_write_molcas_library_no_ecp() {
        let args = BseGetBasisArgsBuilder::default().elements("1-10".to_string()).build().unwrap();
        let basis = get_basis("cc-pVTZ", args);
        let output = write_molcas_library(&basis);
        println!("{output}");
    }
}
