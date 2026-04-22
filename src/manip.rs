//! Common basis set manipulations.
//!
//! This module provides functions for transforming basis sets:
//! - **Uncontracting**: Remove contractions to get primitive shells
//! - **Optimization**: Simplify general contractions
//! - **Augmentation**: Add diffuse or steep functions
//! - **Auxiliary generation**: Create fitting basis sets
//!
//! Most functions modify the basis set in place. Use [`prune_basis`] after
//! manipulations to remove duplicates and zero coefficients.

use crate::prelude::*;

/// Remove duplicate exponents and zero coefficients from a shell.
///
/// Condenses duplicate exponents into general contractions and removes
/// primitives with all-zero coefficients.
///
/// # Arguments
///
/// * `shell` - The electron shell to prune
///
/// # Returns
///
/// A new shell with duplicates removed.
pub fn prune_shell(shell: &BseElectronShell) -> BseElectronShell {
    let mut new_exponents = Vec::new();
    let mut new_coefficients = Vec::new();

    let exponents = &shell.exponents;
    let nprim = exponents.len();

    // transpose of the coefficient matrix
    let coeff_t: Vec<Vec<String>> = (0..shell.coefficients[0].len())
        .map(|i| shell.coefficients.iter().map(|row| row[i].clone()).collect())
        .collect();

    // Group by exponents
    let mut ex_groups: Vec<(String, Vec<Vec<String>>)> = Vec::new();
    for i in 0..nprim {
        let mut found = false;
        for ex in &mut ex_groups {
            if exponents[i].parse::<f64>().unwrap() == ex.0.parse::<f64>().unwrap() {
                ex.1.push(coeff_t[i].clone());
                found = true;
                break;
            }
        }
        if !found {
            ex_groups.push((exponents[i].clone(), vec![coeff_t[i].clone()]));
        }
    }

    // Now collapse within groups
    for ex in ex_groups {
        if ex.1.len() == 1 {
            // only add if there is a nonzero contraction coefficient
            if !ex.1[0].iter().all(|x| x.parse::<f64>().unwrap() == 0.0) {
                new_exponents.push(ex.0);
                new_coefficients.push(ex.1[0].clone());
            }
            continue;
        }

        // ex.1 contains rows of coefficients. The length of ex.1
        // is the number of times the exponent is duplicated. Columns represent general
        // contractions. We want to find the non-zero coefficient in each
        // column, if it exists The result is a single row with a length
        // representing the number of general contractions

        let mut new_coeff_row = Vec::new();

        // so take yet another transpose.
        let ex_coeff: Vec<Vec<String>> =
            (0..ex.1[0].len()).map(|i| ex.1.iter().map(|row| row[i].clone()).collect()).collect();

        for g in ex_coeff {
            let nonzero: Vec<String> = g.iter().filter(|x| x.parse::<f64>().unwrap() != 0.0).cloned().collect();

            if nonzero.len() > 1 {
                panic!("Exponent {} is duplicated within a contraction", ex.0);
            }

            if nonzero.is_empty() {
                new_coeff_row.push(g[0].clone());
            } else {
                new_coeff_row.push(nonzero[0].clone());
            }
        }

        // only add if there is a nonzero contraction coefficient anywhere for this
        // exponent
        if !new_coeff_row.iter().all(|x| x.parse::<f64>().unwrap() == 0.0) {
            new_exponents.push(ex.0);
            new_coefficients.push(new_coeff_row);
        }
    }

    // take the transpose again, putting the general contraction
    // as the slowest index
    let final_coefficients: Vec<Vec<String>> = if !new_coefficients.is_empty() {
        (0..new_coefficients[0].len()).map(|i| new_coefficients.iter().map(|row| row[i].clone()).collect()).collect()
    } else {
        Vec::new()
    };

    BseElectronShell {
        function_type: shell.function_type.clone(),
        region: shell.region.clone(),
        angular_momentum: shell.angular_momentum.clone(),
        exponents: new_exponents,
        coefficients: final_coefficients,
    }
}

/// Remove duplicate shells and zero coefficients from a basis set element.
pub fn prune_basis_in_element(el: &mut BseBasisElement) {
    let shells = match &mut el.electron_shells {
        Some(shells) => shells,
        None => return,
    };

    // Prune each shell
    for shell in shells.iter_mut() {
        *shell = prune_shell(shell);
    }

    // Remove duplicates
    let mut unique_shells = Vec::new();
    for shell in shells.drain(..) {
        if !unique_shells.contains(&shell) {
            unique_shells.push(shell);
        }
    }

    *shells = unique_shells;
}

/// Remove duplicate shells and zero coefficients from a basis set.
///
/// Applies [`prune_shell`] to each shell, then removes duplicate shells.
/// This is meant to be used after other manipulations.
///
/// # Arguments
///
/// * `basis` - The basis set to prune (modified in place)
pub fn prune_basis(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        prune_basis_in_element(el);
    }
}

/// Split combined shells (sp, spd, spdf) into separate shells for an element.
pub fn uncontract_spdf_in_element(el: &mut BseBasisElement, max_am: i32) {
    let Some(electron_shells) = &mut el.electron_shells else {
        return;
    };

    let mut newshells = Vec::new();

    for sh in electron_shells.iter() {
        // am will be a list
        let am = &sh.angular_momentum;
        let coeff = &sh.coefficients;

        // if this is an sp, spd,... orbital
        if am.len() > 1 {
            let mut newsh = sh.clone();
            newsh.angular_momentum = Vec::new();
            newsh.coefficients = Vec::new();

            let ngen = sh.coefficients.len();
            for g in 0..ngen {
                if am[g] > max_am {
                    let mut newsh2 = sh.clone();
                    newsh2.angular_momentum = vec![am[g]];
                    newsh2.coefficients = vec![coeff[g].clone()];
                    newshells.push(newsh2);
                } else {
                    newsh.angular_momentum.push(am[g]);
                    newsh.coefficients.push(coeff[g].clone());
                }
            }

            newshells.insert(0, newsh);
        } else {
            newshells.push(sh.clone());
        }
    }

    *electron_shells = newshells;
}

/// Split combined shells (sp, spd, spdf) into separate shells.
///
/// Combined shells have multiple angular momentum values in a single shell.
/// This function splits them into separate shells, one per angular momentum.
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
/// * `max_am` - Maximum angular momentum to keep combined. For example, if
///   `max_am = 1`, an spd shell becomes sp + d.
///
/// # Note
///
/// The resulting basis may have duplicate shells. Call [`prune_basis`]
/// afterward to clean up.
pub fn uncontract_spdf(basis: &mut BseBasis, max_am: i32) {
    for (_, el) in basis.elements.iter_mut() {
        uncontract_spdf_in_element(el, max_am);
    }
}

/// Remove general contractions from a basis element.
pub fn uncontract_general_in_element(el: &mut BseBasisElement) {
    let Some(ref mut electron_shells) = el.electron_shells else {
        return;
    };

    let mut new_shells = Vec::new();

    for shell in electron_shells.iter() {
        // See if we actually have to uncontract
        // Also, don't uncontract sp, spd,.... orbitals
        //      (leave that to uncontract_spdf)
        if shell.coefficients.len() == 1 || shell.angular_momentum.len() > 1 {
            new_shells.push(shell.clone());
        } else if shell.angular_momentum.len() == 1 {
            for coeff in &shell.coefficients {
                // clone, then replace 'coefficients'
                let mut new_shell = shell.clone();
                new_shell.coefficients = vec![coeff.clone()];
                new_shells.push(new_shell);
            }
        }
    }

    *electron_shells = new_shells;
}

/// Remove general contractions from a basis set.
///
/// General contractions have multiple coefficient vectors for the same
/// set of exponents. This function splits them into separate shells.
/// Shells with all-zero coefficients are removed via [`prune_basis`].
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
///
/// # Note
///
/// Combined shells (sp, spd, etc.) are not affected. Use [`uncontract_spdf`]
/// for those.
pub fn uncontract_general(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        uncontract_general_in_element(el);
    }
    prune_basis(basis);
}

/// Fully uncontract a basis element to individual primitives
pub fn uncontract_segmented_in_element(el: &mut BseBasisElement) {
    let Some(ref mut electron_shells) = el.electron_shells else {
        return;
    };

    let mut new_shells = Vec::new();

    for shell in electron_shells.iter() {
        let exponents = &shell.exponents;
        let nam = shell.angular_momentum.len();

        for exponent in exponents.iter() {
            let mut new_shell = shell.clone();
            new_shell.exponents = vec![exponent.clone()];
            new_shell.coefficients = vec![vec!["1.00000000E+00".to_string(); nam]];

            // Transpose the coefficients
            new_shell.coefficients = misc::transpose_matrix(&new_shell.coefficients);

            new_shells.push(new_shell);
        }
    }

    *electron_shells = new_shells;
}

/// Fully uncontract a basis set to individual primitives.
///
/// Each primitive becomes a separate shell with coefficient 1.0.
/// Combined shells (sp, spd, etc.) are preserved as separate shells
/// for each angular momentum.
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
pub fn uncontract_segmented(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        uncontract_segmented_in_element(el);
    }
}

/// Combine shells of the same angular momentum into general contractions for a
/// basis element.
pub fn make_general_in_element(el: &mut BseBasisElement) {
    const ZERO: &str = "0.00000000";

    let electron_shells = match &mut el.electron_shells {
        Some(shells) => shells,
        None => return,
    };

    let mut newshells = Vec::new();

    // See what we have
    let mut all_am = Vec::new();
    for sh in electron_shells.iter() {
        let am = &sh.angular_momentum;

        // Skip sp shells
        if am.len() > 1 {
            newshells.push(sh.clone());
            continue;
        }

        if !all_am.contains(am) {
            all_am.push(am.clone());
        }
    }

    all_am.sort();

    for am in all_am {
        let mut newsh = BseElectronShell {
            angular_momentum: am.clone(),
            exponents: Vec::new(),
            coefficients: Vec::new(),
            region: String::new(),
            function_type: String::new(),
        };

        // Do exponents first
        for sh in electron_shells.iter() {
            if sh.angular_momentum == am {
                newsh.exponents.extend(sh.exponents.clone());
            }
        }

        // Number of primitives in the new shell
        let nprim = newsh.exponents.len();

        let mut cur_prim = 0;
        for sh in electron_shells.iter() {
            if sh.angular_momentum != am {
                continue;
            }

            if newsh.function_type.is_empty() {
                newsh.function_type = sh.function_type.clone();
            }

            // Make sure the shells we are merging have the same function types
            let ft1 = &newsh.function_type;
            let ft2 = &sh.function_type;

            // Check if one function type is the subset of another
            // (should handle gto/gto_spherical, etc)
            if !ft1.contains(ft2) && !ft2.contains(ft1) {
                panic!("Cannot make general contraction of different function types");
            }

            let ngen = sh.coefficients.len();

            for g in 0..ngen {
                let mut coef = vec![ZERO.to_string(); cur_prim];
                coef.extend(sh.coefficients[g].clone());
                coef.extend(vec![ZERO.to_string(); nprim - coef.len()]);
                newsh.coefficients.push(coef);
            }

            cur_prim += sh.exponents.len();
        }

        newshells.push(newsh);
    }

    el.electron_shells = Some(newshells);
}

/// Combine shells of the same angular momentum into general contractions.
///
/// Creates one large general contraction for each angular momentum per element.
/// This is the inverse of [`uncontract_general`].
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
/// * `skip_spdf` - If true, leave combined shells (sp, spd) untouched
///
/// # Note
///
/// The output may not be aesthetically pleasing. Use
/// [`sort_basis`][crate::sort::sort_basis] afterward for better formatting.
pub fn make_general(basis: &mut BseBasis, skip_spdf: bool) {
    if !skip_spdf {
        uncontract_spdf(basis, 0);
    }

    for (_, el) in basis.elements.iter_mut() {
        make_general_in_element(el);
    }

    // If the basis was read in from a segmented format, it will have
    // duplicate primitives, and so a pruning is necessary
    prune_basis(basis);
}

/// Check if a coefficient column has exactly one non-zero value.
///
/// Used to identify free (uncontracted) primitives.
fn is_single_column(col: &[String]) -> bool {
    col.iter().filter(|s| s.parse::<f64>().unwrap() != 0.0).count() == 1
}

/// Find indices of free (uncontracted) primitives in a coefficient matrix.
fn free_primitives(coeffs: &[Vec<String>]) -> Vec<usize> {
    // Find which columns represent free primitives
    let single_columns = coeffs.iter().filter(|col| is_single_column(col)).collect_vec();
    if single_columns.is_empty() {
        return vec![];
    }

    // Now dig out the functions on those columns
    let mut csum = vec![0.0; single_columns[0].len()];
    for col in &single_columns {
        for k in 0..col.len() {
            csum[k] += col[k].parse::<f64>().unwrap();
        }
    }

    // Since we're only looking at columns that represent free
    // primitives, the rows that have non-zero sums correspond to free
    // exponents.
    csum.iter().enumerate().filter(|(_, val)| **val != 0.0).map(|(idx, _)| idx).collect_vec()
}

/// Remove uncontracted (free) primitives from a basis element.
pub fn remove_free_primitives_in_element(el: &mut BseBasisElement) {
    let Some(ref mut electron_shells) = el.electron_shells else {
        return;
    };

    let mut new_shells = Vec::new();
    for sh in electron_shells.iter_mut() {
        // find contractions
        let coefficients = sh.coefficients.iter().filter(|&col| !is_single_column(col)).cloned().collect_vec();
        if !coefficients.is_empty() {
            sh.coefficients = coefficients;
            new_shells.push(sh.clone());
        }
    }
    el.electron_shells = Some(new_shells);
}

/// Remove uncontracted (free) primitives from a basis set.
///
/// Free primitives have coefficient 1.0 in only one contraction.
/// This can be used to generate a minimal basis from an augmented set.
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
pub fn remove_free_primitives(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        if el.electron_shells.is_none() {
            continue;
        }

        remove_free_primitives_in_element(el);
    }
    // TODO: prune_basis
}

/// Optimizes the general contraction using the method of Hashimoto et al for a
/// basis element.
pub fn optimize_general_in_element(el: &mut BseBasisElement) {
    if el.electron_shells.is_none() {
        return;
    }

    let elshells = el.electron_shells.as_mut().unwrap();
    for sh in elshells {
        let coefficients = &mut sh.coefficients;
        let nam = sh.angular_momentum.len();

        // Skip sp shells and shells with only one general contraction
        if nam > 1 || coefficients.len() < 2 {
            continue;
        }

        // First, find columns (general contractions) with a single non-zero value
        let single_columns: Vec<usize> =
            coefficients.iter().enumerate().filter(|(_, c)| is_single_column(c)).map(|(idx, _)| idx).collect();

        // Find the corresponding rows that have a value in one of these columns
        // Note that at this stage, the row may have coefficients in more than one
        // column. That is what we are looking for

        // Also, test to see that each row is only represented once. That is, there
        // should be no rows that are part of single columns (this would
        // represent duplicate shells). This can happen in poorly-formatted
        // basis sets and is an error
        let mut row_col_pairs = Vec::new();
        let mut all_row_idx = Vec::new();
        for &col_idx in &single_columns {
            let col = &coefficients[col_idx];
            col.iter().enumerate().for_each(|(row_idx, value)| {
                if value.parse::<f64>().unwrap() != 0.0 && !all_row_idx.contains(&row_idx) {
                    // Store the index of the nonzero value in single_columns
                    row_col_pairs.push((row_idx, col_idx));
                    all_row_idx.push(row_idx);
                }
            });
        }

        // Now for each row/col pair, zero out the entire row
        // EXCEPT for the column that has the single value
        for (row_idx, col_idx) in row_col_pairs {
            for (idx, col) in coefficients.iter_mut().enumerate() {
                if col[row_idx].parse::<f64>().unwrap() != 0.0 && col_idx != idx {
                    col[row_idx] = "0.0000000E+00".to_string();
                }
            }
        }
    }
}

/// Optimizes the general contraction using the method of Hashimoto et al
///
/// # See also
///
/// T. Hashimoto, K. Hirao, H. Tatewaki
/// 'Comment on Dunning's correlation-consistent basis set'
/// Chemical Physics Letters v243, Issues 1-2, pp, 190-192 (1995)
/// <https://doi.org/10.1016/0009-2614(95)00807-G>
pub fn optimize_general(basis: &mut BseBasis) {
    // Make as generally-contracted as possible first
    make_general(basis, true);

    for (_, el) in basis.elements.iter_mut() {
        optimize_general_in_element(el);
    }
}

/// Add diffuse or steep functions via even-tempered extrapolation in a basis
/// element.
pub fn geometric_augmentation_in_element(el: &mut BseBasisElement, nadd: i32, steep: bool) {
    let eldata = el.clone();

    let Some(electron_shells) = &eldata.electron_shells else {
        return;
    };

    let mut new_shells = Vec::new();

    for shell in electron_shells {
        // Find the two smallest exponents. The smallest is alpha
        // beta is the ratio alpha/(next smallest)
        // Keep track of the indices as well
        let mut exponents: Vec<(f64, usize)> =
            shell.exponents.iter().enumerate().map(|(idx, x)| (x.parse::<f64>().unwrap(), idx)).collect();

        exponents.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if exponents.len() < 2 {
            // Need at least two exponents to perform augmentation
            continue;
        }

        let (ref_idx, next_idx) = if steep {
            // If we're augmenting by steep functions, the
            // references are the steepest and second-steepest
            // function.
            (exponents.len() - 1, exponents.len() - 2)
        } else {
            // If we're augmenting by diffuse functions, the
            // references are the diffusemost and second-most
            // diffuse function.
            (0, 1)
        };

        let (ref_exp, ref_idx) = exponents[ref_idx];
        let (next_exp, next_idx) = exponents[next_idx];
        // Even-tempered spacing parameter
        let beta = ref_exp / next_exp;

        if (ref_exp - next_exp).abs() < f64::EPSILON {
            panic!(
                    "The two outermost exponents are the same. Duplicate exponents are not a good thing here. Exponent: {ref_exp}"
                );
        }

        // Test that the primitives for the references are free.
        let free_prims = free_primitives(&shell.coefficients);
        if !free_prims.contains(&ref_idx) || !free_prims.contains(&next_idx) {
            // The shell does not have enough free primitives so
            // skip the extrapolation.
            continue;
        }

        // Form new exponents
        let mut new_exponents = Vec::new();
        for i in 1..=nadd {
            new_exponents.push(ref_exp * beta.powi(i));
        }

        let new_exponents: Vec<String> = new_exponents.iter().map(|&x| misc::format_exponent(x)).collect();

        // add the new exponents as new uncontracted shells
        for ex in new_exponents {
            new_shells.push(BseElectronShell {
                function_type: shell.function_type.clone(),
                region: shell.region.clone(),
                angular_momentum: shell.angular_momentum.clone(),
                exponents: vec![ex],
                coefficients: vec![vec!["1.00000000".to_string()]],
            });
        }
    }

    // add the shells to the original basis set
    if el.electron_shells.is_none() {
        el.electron_shells = Some(Vec::new());
    }
    if let Some(shells) = &mut el.electron_shells {
        shells.extend(new_shells);
    }
}

/// Add diffuse or steep functions via even-tempered extrapolation.
///
/// Extends a basis set by adding new functions using geometric progression.
/// For augmented Dunning sets (aug), diffuse augmentation corresponds to
/// multiple augmentation (aug -> daug, taug, ...).
///
/// # Arguments
///
/// * `basis` - The basis set to modify (modified in place)
/// * `nadd` - Number of functions to add (must be >= 1)
/// * `steep` - If true, add steep functions; if false, add diffuse functions
///
/// # Requirements
///
/// The two outermost primitives must be free (uncontracted) for the
/// extrapolation to work correctly.
///
/// # Reference
///
/// Woon & Dunning, Jr., J. Chem. Phys. 100, 2975 (1994)
/// <https://doi.org/10.1063/1.466439>
pub fn geometric_augmentation(basis: &mut BseBasis, nadd: i32, steep: bool) {
    if nadd < 1 {
        panic!("Adding {nadd} functions makes no sense for geometric_augmentation");
    }

    // We need to combine shells by AM
    // make_general is assumed to be implemented elsewhere
    make_general(basis, true);

    // From Woon & Dunning, Jr
    // J. Chem. Phys. v100, No. 4, p. 2975 (1994)
    // DOI: 10.1063/1.466439
    //
    // The exponent for d-aug-cc-pVXZ: alpha*beta
    //                  t-aug-cc-pVXZ: alpha*(beta**2)
    // and so on.
    //
    // alpha = smallest exponent in aug-cc-pVXZ
    // beta = ratio of two most diffuse functions (beta < 1)
    //
    // This applies to all angular momentum (which have been combined into
    // shells already)

    for (_, el) in basis.elements.iter_mut() {
        geometric_augmentation_in_element(el, nadd, steep);
    }
}

/// Create a Coulomb fitting basis set for the given orbital basis set.
///
/// # See also
///
/// G. L. Stoychev, A. A. Auer, and F. Neese
/// 'Automatic Generation of Auxiliary Basis Sets'
/// J. Chem. Theory Comput. 13, 554 (2017)
/// <http://doi.org/10.1021/acs.jctc.6b01041>
pub fn autoaux_basis(basis: &BseBasis) -> BseBasis {
    use libm::tgamma;
    use std::f64::consts::PI;

    // We want the basis set as generally contracted. Get a copy so
    // that we don't change the input set
    let mut basis = basis.clone();
    make_general(&mut basis, false);

    let mut auxbasis_data = HashMap::new();

    for (element_Z, eldata) in &basis.elements {
        let elshells = match &eldata.electron_shells {
            Some(shells) => shells,
            None => {
                println!("No electron shells for {element_Z}");
                continue;
            },
        };

        // What is maximal angular momentum?
        let lmax = elshells.iter().map(|sh| sh.angular_momentum[0]).max().unwrap_or(0);

        fn update_minimum_array(array: &mut [Option<f64>], index: usize, value: f64) {
            if let Some(current) = array[index] {
                array[index] = Some(current.min(value));
            } else {
                array[index] = Some(value);
            }
        }

        fn update_maximum_array(array: &mut [Option<f64>], index: usize, value: f64) {
            if let Some(current) = array[index] {
                array[index] = Some(current.max(value));
            } else {
                array[index] = Some(value);
            }
        }

        // Form values of smallest and largest primitive exponent
        let mut amin = vec![None; lmax as usize + 1];
        let mut amax_prim = vec![None; lmax as usize + 1];
        let mut amax_eff = vec![None; lmax as usize + 1];

        for sh in elshells {
            let exponents = &sh.exponents;
            let coefficients = &sh.coefficients;
            let ncontr = coefficients.len();
            let shell_am = &sh.angular_momentum;
            assert_eq!(shell_am.len(), 1);
            let l = shell_am[0] as usize;

            // Store values of smallest and largest exponent
            let mut expval: Vec<f64> = exponents.iter().map(|x| x.parse().unwrap()).collect();
            expval.sort_by(|a, b| b.partial_cmp(a).unwrap());

            update_maximum_array(&mut amax_prim, l, expval[0]);
            update_minimum_array(&mut amin, l, *expval.last().unwrap());

            // Now we just compute the spatial extent <r> for functions (in contracted
            // form), eq (8) in the paper
            let rmat = ints::gto_R_contr(exponents, coefficients, shell_am[0]);
            // Extract the diagonal values
            let rvec: Vec<f64> = (0..ncontr).map(|i| rmat[i][i]).collect();

            // This gives us the "quasi-uncontracted" orbital basis with primitive exponents
            // Prefactor defined in eq 10
            let k_value = 2f64.powi(2 * l as i32 + 1) * tgamma(l as f64 + 2.0).powi(2) / tgamma(2.0 * l as f64 + 3.0);

            // Calculate effective exponent with eq 9, note that it
            // must be proportional to the inverse square of the
            // radius, not the inverse radius
            let effective_exponents: Vec<f64> =
                rvec.iter().map(|rexp| 2.0 * k_value.powi(2) / (PI * rexp.powi(2))).collect();

            // Sort list in decreasing order
            let mut effective_exponents = effective_exponents;
            effective_exponents.sort_by(|a, b| b.partial_cmp(a).unwrap());
            // Store largest effective exponent
            update_maximum_array(&mut amax_eff, l, effective_exponents[0]);
        }

        // Collect the smallest and largest exponents
        let mut a_minaux = vec![None; 2 * lmax as usize + 1];
        let mut a_maxaux_prim = vec![None; 2 * lmax as usize + 1];
        let mut a_maxaux_eff = vec![None; 2 * lmax as usize + 1];

        for l in 0..=lmax as usize {
            for lp in l..=lmax as usize {
                // Calculate the values of the exponents
                let minaux = amin[l].unwrap() + amin[lp].unwrap();
                let maxauxp = amax_prim[l].unwrap() + amax_prim[lp].unwrap();
                let maxauxe = amax_eff[l].unwrap() + amax_eff[lp].unwrap();

                // Loop over all possible coupled angular momenta
                for laux in ((l as i32 - lp as i32).unsigned_abs() as usize)..=(l + lp) {
                    update_minimum_array(&mut a_minaux, laux, minaux);
                    update_maximum_array(&mut a_maxaux_prim, laux, maxauxp);
                    update_maximum_array(&mut a_maxaux_eff, laux, maxauxe);
                }
            }
        }

        // Form lval: highest occupied momentum of occupied shells for
        // atom. H and He have lval=0; Li, Be and everything after that
        // have lval=1; 3d transition metals have lval=2 and
        // lanthanoids have lval=3.
        let z = element_Z.parse::<i32>().unwrap();
        let lval = match z {
            1..=2 => 0,   // H and He
            3..=20 => 1,  // Li - Ca
            21..=56 => 2, // Sc - Ba
            57.. => 3,
            ..=0 => unreachable!(),
        };

        // Form linc: 1 up to Ar, 2 for the rest
        let linc = if z > 18 { 2 } else { 1 };

        // Limit maximal angular momentum
        let lmax_aux = (2 * lval).max((lmax + linc) as i32).min(2 * lmax) as usize;
        println!("Generating auxiliary basis for element {element_Z} with lmax_aux = {lmax_aux}");

        // Values from Table I; factor 7.0 for P functions is missing in the paper
        let flaux = [20.0, 7.0, 4.0, 4.0, 3.5, 2.5, 2.0, 2.0];
        let blaux_big = [1.8, 2.0, 2.2, 2.2, 2.2, 2.3, 3.0, 3.0];
        let b_small = 1.8;

        // Form actual upper limit for even-tempered expansion
        let mut amax_aux = vec![None; lmax_aux + 1];
        for laux in 0..=lmax_aux {
            if laux <= 2 * lval as usize {
                // There's a typo in the paper, max instead of min
                amax_aux[laux] = Some((flaux[laux] * a_maxaux_eff[laux].unwrap()).min(a_maxaux_prim[laux].unwrap()));
            } else {
                amax_aux[laux] = Some(a_maxaux_eff[laux].unwrap());
            }
        }

        // Create aux basis
        let aux_element_data = auxbasis_data.entry(element_Z.to_string()).or_insert_with(BseBasisElement::default);

        for laux in 0..=lmax_aux {
            // Generate the exponents
            let mut exponents = Vec::new();
            let mut current_exponent = a_minaux[laux].unwrap();
            loop {
                exponents.push(misc::format_exponent(current_exponent));
                // This is not exactly the same to original Python bse implementation
                // where its condition is `current_exponent >= amax_aux[laux]`
                // `amax_aux` can suffer some numerical discrepancies; so a little resolve here
                if current_exponent - amax_aux[laux].unwrap() > 10.0 * f64::EPSILON {
                    break;
                }

                if laux <= 2 * lval as usize {
                    current_exponent *= b_small;
                } else {
                    current_exponent *= blaux_big[laux.min(blaux_big.len() - 1)];
                }
            }

            // Create shells
            for z in exponents {
                let function_type = lut::function_type_from_am(&[laux as i32], "gto", "spherical");
                let shell = BseElectronShell {
                    function_type,
                    region: String::new(),
                    angular_momentum: vec![laux as i32],
                    exponents: vec![z],
                    coefficients: vec![vec!["1.0".to_string()]],
                };
                aux_element_data.electron_shells.get_or_insert_with(Default::default).push(shell);
            }
        }
    }

    // Finalize basis
    let mut molssi_bse_schema = basis.molssi_bse_schema.clone();
    molssi_bse_schema.schema_type = "component".to_string();
    let function_types = compose::whole_basis_types(&auxbasis_data);

    BseBasis {
        molssi_bse_schema,
        revision_description: basis.revision_description.clone(),
        revision_date: basis.revision_date.clone(),
        elements: auxbasis_data,
        version: basis.version.clone(),
        function_types,
        names: basis.names.clone(),
        tags: basis.tags.clone(),
        family: basis.family.clone(),
        description: String::new(),
        role: "rifit".to_string(),
        auxiliaries: HashMap::new(),
        name: format!("{}_autoaux", basis.name),
    }
}

/// Create a Coulomb fitting basis set for the given orbital basis set.
///
/// # See also
///
/// R. Yang, A. P. Rendell, and M. J. Frisch
/// 'Automatically generated Coulomb fitting basis sets:
/// Design and accuracy for systems containing H to Kr'
/// J. Chem. Phys. 127, 074102 (2007)
/// <http://doi.org/10.1063/1.2752807>
pub fn autoabs_basis(basis: &BseBasis, lmaxinc: i32, fsam: f64) -> BseBasis {
    // We want the basis set as generally contracted. Get a copy so
    // that we don't change the input set
    let mut basis_copy = basis.clone();
    make_general(&mut basis_copy, false);

    let mut auxbasis_data = HashMap::new();

    for (element_Z, eldata) in &basis_copy.elements {
        let elshells = match &eldata.electron_shells {
            Some(shells) => shells,
            None => {
                println!("No electron shells for {element_Z}");
                continue;
            },
        };

        // Form the list of candidate functions
        let mut candidates = Vec::new();
        for sh in elshells {
            let exponents = &sh.exponents;
            let shell_am = &sh.angular_momentum;
            assert_eq!(shell_am.len(), 1);
            for x in exponents {
                // We do the doubling here
                let exponent = x.parse::<f64>().unwrap() * 2.0;
                candidates.push((exponent, shell_am[0]));
            }
        }

        // Form lval: highest occupied momentum of occupied shells for
        // atom. H and He have lval=0; Li, Be and everything after that
        // have lval=1; 3d transition metals have lval=2 and
        // lanthanoids have lval=3.
        let z = element_Z.parse::<i32>().unwrap();
        let lval = if z > 54 {
            3
        } else if z > 18 {
            2
        } else if z > 2 {
            1
        } else {
            0
        };

        // Maximal candidate am
        let lmax = candidates.iter().map(|c| c.1).max().unwrap_or(0);

        // Maximal allowed angular momentum
        let lmax_aux = (2 * lval).max(lmax + lmaxinc).min(2 * lmax);

        // Fitting functions
        let mut fit_functions = Vec::new();

        while !candidates.is_empty() {
            // Sort candidates by exponent
            candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Move top candidate to trial function set
            let mut trial_functions = vec![candidates.pop().unwrap()];
            while let Some(candidate) = candidates.last() {
                // trial fitting functions for which the ratio of the
                // exponent reference value divided by the value of
                // their exponent is smaller than fsam are moved from
                // the candidate basis set to the trail function set
                if trial_functions[0].0 / candidate.0 < fsam {
                    trial_functions.push(candidates.pop().unwrap());
                } else {
                    break;
                }
            }

            // Calculate geometric average of functions in trial set
            let log_sum: f64 = trial_functions.iter().map(|tr| tr.0.ln()).sum();
            let average_exponent = (log_sum / trial_functions.len() as f64).exp();

            // The angular moment of this function is set to the
            // maximum angular moment of any primitive in the current
            // trial set and the previous ABSs.
            let mut max_fit_am = fit_functions.iter().map(|f: &(f64, i32)| f.1).max().unwrap_or(0);
            max_fit_am = max_fit_am.max(trial_functions.iter().map(|f| f.1).max().unwrap_or(0));
            // Reset to lmax_aux
            max_fit_am = max_fit_am.min(lmax_aux);

            // Add functions
            for fit_am in 0..=max_fit_am {
                fit_functions.push((average_exponent, fit_am));
            }
        }

        // Create aux basis
        let aux_element_data = auxbasis_data.entry(element_Z.to_string()).or_insert_with(BseBasisElement::default);

        // Create shells
        for f in fit_functions {
            let func_type = lut::function_type_from_am(&[f.1], "gto", "spherical");
            let shell = BseElectronShell {
                function_type: func_type,
                region: String::new(),
                angular_momentum: vec![f.1],
                exponents: vec![misc::format_exponent(f.0)],
                coefficients: vec![vec!["1.0".to_string()]],
            };
            aux_element_data.electron_shells.get_or_insert_with(Default::default).push(shell);
        }
    }

    // Finalize basis
    let mut molssi_bse_schema = basis.molssi_bse_schema.clone();
    molssi_bse_schema.schema_type = "component".to_string();
    let function_types = compose::whole_basis_types(&auxbasis_data);

    BseBasis {
        molssi_bse_schema,
        revision_description: basis.revision_description.clone(),
        revision_date: basis.revision_date.clone(),
        elements: auxbasis_data,
        version: basis.version.clone(),
        function_types,
        names: basis.names.clone(),
        tags: basis.tags.clone(),
        family: basis.family.clone(),
        description: String::new(),
        role: "jfit".to_string(),
        auxiliaries: HashMap::new(),
        name: format!("{}_autoabs", basis.name),
    }
}

/// Remove a primitive (by index) from an electron shell.
///
/// Modifies the shell in place, removing the exponent at the given index
/// and its corresponding coefficients from all contraction vectors.
/// Also removes general contractions if all coefficients become zero.
fn remove_primitive(shell: &mut BseElectronShell, idx_to_remove: usize) {
    shell.exponents.remove(idx_to_remove);
    for coeff in shell.coefficients.iter_mut() {
        coeff.remove(idx_to_remove);
    }

    // Remove general contractions if all coefficients are zero
    shell.coefficients =
        shell.coefficients.iter().filter(|gen| gen.iter().any(|c| c.parse::<f64>().unwrap() != 0.0)).cloned().collect();
}

/// Enum specifying how many diffuse functions to remove.
enum DiffuseRemoval {
    /// Remove all diffuse functions.
    All,
    /// Remove a specific number of diffuse functions.
    Count(i32),
}

/// Remove diffuse functions from an element's electron shells.
///
/// Removes the most diffuse primitive(s) from shells with the highest
/// angular momenta. The number of diffuse functions to remove is specified
/// by `nremove`.
///
/// # Arguments
///
/// * `eldata` - The element data containing electron shells (modified in place)
/// * `nremove` - Number of diffuse functions to remove. Can be a number or
///   "all" to remove all diffuse functions.
fn element_remove_diffuse(eldata: &mut BseBasisElement, nremove: DiffuseRemoval) {
    let Some(shells) = &mut eldata.electron_shells else {
        return;
    };

    let max_am = misc::max_am(shells);
    let nremove_val = match nremove {
        DiffuseRemoval::All => max_am + 1,
        DiffuseRemoval::Count(n) => n,
    };

    // Build list of angular momenta to remove (from highest to lowest)
    let am_toremove: Vec<i32> = (max_am - nremove_val + 1..=max_am).filter(|x| *x >= 0).collect();

    for shell in shells.iter_mut() {
        let shell_am = &shell.angular_momentum;

        // Cannot remove diffuse from fused shell (sp, spd, etc)
        if shell_am.len() > 1 {
            panic!("Cannot remove diffuse functions from fused shell: {:?}", shell_am);
        }

        let shell_am_val = shell_am[0];
        if !am_toremove.contains(&shell_am_val) {
            continue;
        }

        // Find the most diffuse function and remove it
        let exponents: Vec<(f64, usize)> =
            shell.exponents.iter().enumerate().map(|(idx, x)| (x.parse::<f64>().unwrap(), idx)).collect();

        // Sort by exponent (ascending - smallest is most diffuse)
        let mut sorted_exponents = exponents;
        sorted_exponents.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Get the index of the most diffuse exponent
        let (_, diffuse_idx) = sorted_exponents[0];

        remove_primitive(shell, diffuse_idx);
    }
}

/// Create the Truhlar "calendar" basis sets from an augmented basis set.
///
/// The Truhlar calendar basis sets (jul, jun, may, apr, mar, feb, jan) are
/// derived from augmented Dunning basis sets by progressively removing
/// diffuse functions.
///
/// The naming convention uses months, going backwards through the calendar:
/// - `jul` (July): Remove all diffuse functions from H, He only
/// - `jun` (June): Remove 1 diffuse function from all elements (except H, He)
/// - `may` (May): Remove 2 diffuse functions from all elements (except H, He)
/// - ... and so on
///
/// # Arguments
///
/// * `basis` - The augmented basis set to modify
/// * `month` - The target month ("jul", "jun", "may", "apr", "mar", "feb",
///   "jan")
///
/// # Returns
///
/// A new basis set with the appropriate diffuse functions removed.
///
/// # Errors
///
/// Returns an error if the month is invalid or if too many diffuse functions
/// would be removed for the given basis set.
///
/// # See also
///
/// E. Papajak, J. Zheng, X. Xu, H. R. Leverentz, D. G. Truhlar
/// 'Assessment of the accuracy of density functionals for prediction
/// of relative energies and geometries of optimized structures'
/// J. Chem. Theory Comput. 7, 3027 (2011)
/// <https://doi.org/10.1021/ct200106n>
pub fn truhlar_calendarize(basis: &BseBasis, month: &str) -> Result<BseBasis, BseError> {
    let valid_months = ["jul", "jun", "may", "apr", "mar", "feb", "jan"];
    let month_lower = month.to_lowercase();
    if !valid_months.contains(&month_lower.as_str()) {
        return bse_raise!(ValueError, "Month '{}' is not valid for truhlar calendarization", month);
    }

    // Make a copy and make general contractions
    let mut basis = basis.clone();
    make_general(&mut basis, false);

    // Find the max am for the entire basis
    let all_am: Vec<i32> = basis
        .elements
        .values()
        .filter_map(|el| el.electron_shells.as_ref())
        .map(|shells| misc::max_am(shells))
        .collect();
    let basis_max_am = all_am.iter().max().copied().unwrap_or(0);

    // Figure out what to remove
    // The index represents the offset from jul (index('jul') = 0)
    // This offset represents how many diffuse functions to remove from each element
    let month_offset = valid_months.iter().position(|m| *m == month_lower.as_str()).unwrap() as i32;

    // Check that we're not removing too many diffuse functions
    // For a given basis_max_am, there will be (basis_max_am+1) diffuse functions
    if month_offset > basis_max_am {
        return bse_raise!(
            ValueError,
            "Will not remove {} diffuse functions for basis with max am = {}",
            month_offset,
            basis_max_am
        );
    }

    // jul = same as aug, except remove all diffuse from H,He
    // First, remove diffuse functions from H,He (this always happens)
    for el_z in ["1", "2"] {
        if let Some(eldata) = basis.elements.get_mut(el_z) {
            element_remove_diffuse(eldata, DiffuseRemoval::All);
        }
    }

    // Short circuit if we are doing jul
    if month_offset == 0 {
        prune_basis(&mut basis);
        return Ok(basis);
    }

    // Now handle the rest of the elements
    for (el_z, eldata) in basis.elements.iter_mut() {
        if el_z == "1" || el_z == "2" {
            continue;
        }

        element_remove_diffuse(eldata, DiffuseRemoval::Count(month_offset));
    }

    prune_basis(&mut basis);
    Ok(basis)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_prune_shell() {
        let shell = BseElectronShell {
            function_type: "gto".to_string(),
            region: "1".to_string(),
            angular_momentum: vec![0, 1],
            exponents: vec!["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
            coefficients: vec![vec!["0.2".to_string(), "0.5".to_string(), "0.0".to_string()]],
        };
        let pruned_shell = prune_shell(&shell);
        assert_eq!(pruned_shell.exponents, vec!["1.0".to_string(), "2.0".to_string()]);
        assert_eq!(pruned_shell.coefficients, vec![vec!["0.2".to_string(), "0.5".to_string()]]);
    }
}
