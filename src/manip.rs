//! Common basis set manipulations
//!
//! This module contains functions for uncontracting and merging basis set data,
//! as well as some other small functions.

use crate::prelude::*;

/// Removes exact duplicates of primitives, and condenses duplicate exponents
/// into general contractions.
///
/// Also removes primitives if all coefficients are zero.
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

pub fn prune_basis(basis: &mut BseBasis) {
    // Removes primitives that have a zero coefficient, and
    // removes duplicate primitives and shells
    //
    // This only finds EXACT duplicates, and is meant to be used
    // after other manipulations

    for (_, el) in basis.elements.iter_mut() {
        let shells = match &mut el.electron_shells {
            Some(shells) => shells,
            None => continue,
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
}

/// Removes sp, spd, spdf, etc, contractions from a basis set.
///
/// The general contractions are replaced by uncontracted versions
///
/// Contractions up to max_am will be left in place. For example,
/// if max_am = 1, spd will be split into sp and d
///
/// The input basis set is modified directly.
/// The returned basis may have functions with coefficients of zero and may
/// have duplicate shells.
pub fn uncontract_spdf(basis: &mut BseBasis, max_am: i32) {
    for (_, el) in basis.elements.iter_mut() {
        let Some(electron_shells) = &mut el.electron_shells else {
            continue;
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
}

/// Removes the general contractions from a basis set
///
/// The input basis set is modified in place. The resulting basis
/// may have functions with coefficients of zero and may have duplicate
/// shells.
pub fn uncontract_general(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        let Some(ref mut electron_shells) = el.electron_shells else {
            continue;
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
}

/// Removes the segmented contractions from a basis set
///
/// This implicitly removes general contractions as well,
/// but will leave sp, spd, ... orbitals alone
///
/// The input basis set is modified directly.
/// The resulting basis may have functions with coefficients of zero
/// and may have duplicate shells.
pub fn uncontract_segmented(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        let Some(electron_shells) = &mut el.electron_shells else {
            continue;
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
}

/// Makes one large general contraction for each angular momentum
///
/// The output of this function is not pretty. If you want to make it nicer,
/// use sort_basis afterwards.
pub fn make_general(basis: &mut BseBasis, skip_spdf: bool) {
    let zero = "0.00000000";

    if !skip_spdf {
        uncontract_spdf(basis, 0);
    }

    for (_, el) in basis.elements.iter_mut() {
        let electron_shells = match &mut el.electron_shells {
            Some(shells) => shells,
            None => continue,
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
                    let mut coef = vec![zero.to_string(); cur_prim];
                    coef.extend(sh.coefficients[g].clone());
                    coef.extend(vec![zero.to_string(); nprim - coef.len()]);
                    newsh.coefficients.push(coef);
                }

                cur_prim += sh.exponents.len();
            }

            newshells.push(newsh);
        }

        el.electron_shells = Some(newshells);
    }

    // If the basis was read in from a segmented format, it will have
    // duplicate primitives, and so a pruning is necessary
    prune_basis(basis);
}

/// Check if there is only one non-zero coefficient in a vector.
/// This function is used to determine if two basis shares the same exponents
/// but different coefficients.
fn is_single_column(col: &[String]) -> bool {
    col.iter().filter(|s| s.parse::<f64>().unwrap() != 0.0).count() == 1
}

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

/// Removes any free primitives from a basis set as a way to generate a minimal
/// basis.
///
/// The input basis set is not modified. The returned basis may have functions
/// with coefficients of zero and may have duplicate shells.
pub fn remove_free_primitives(basis: &mut BseBasis) {
    for (_, el) in basis.elements.iter_mut() {
        if el.electron_shells.is_none() {
            continue;
        }

        let mut new_shells: Vec<BseElectronShell> = vec![];
        for sh in el.electron_shells.as_mut().unwrap() {
            // find contractions
            let coefficients = sh.coefficients.iter().filter(|&col| !is_single_column(col)).cloned().collect_vec();
            if !coefficients.is_empty() {
                sh.coefficients = coefficients;
                new_shells.push(sh.clone());
            }
        }
        el.electron_shells = Some(new_shells);
    }

    // TODO: prune_basis
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

    for (_, eldata) in basis.elements.iter_mut() {
        if eldata.electron_shells.is_none() {
            continue;
        }

        let elshells = eldata.electron_shells.as_mut().unwrap();
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
}

/// Extends a basis set by adding extrapolated diffuse or steep functions.
///
/// For augmented Dunning sets (aug), the diffuse augmentation
/// corresponds to multiple augmentation (aug -> daug, taug, ...).
///
/// In order for the augmentation to make sense, the two outermost
/// primitives have to be free i.e. uncontracted.
///
/// Parameters
/// ----------
/// basis: &mut BseBasis
///     Basis set dictionary to work with
/// nadd: i32
///     Number of functions to add (must be >=1). For diffuse augmentation on an
///     augmented set: 1 -> daug, 2 -> taug, etc use_copy: bool
///     If True, the input basis set is not modified.
/// steep: bool
///     If True, the augmentation is done for steep functions instead of diffuse
///     functions.
pub fn geometric_augmentation(basis: &mut BseBasis, nadd: i32, steep: bool) {
    if nadd < 1 {
        panic!("Adding {nadd} functions makes no sense for geometric_augmentation");
    }

    // We need to combine shells by AM
    // make_general is assumed to be implemented elsewhere
    let mut basis_copy = basis.clone();
    make_general(&mut basis_copy, true);

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

    for (el_z, eldata) in basis_copy.elements.iter() {
        let Some(electron_shells) = &eldata.electron_shells else {
            continue;
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

            // the scientific notation of rust is not exactly the same as Python's
            #[inline]
            fn format_exponent(exp: f64) -> String {
                let token = format!("{exp:.6e}");
                let (s, e) = token.split_once('e').unwrap();
                let e = e.parse::<i32>().unwrap();
                let sgn = if e < 0 { '-' } else { '+' };
                let e = e.abs();
                format!("{s}e{sgn}{e:0>2}")
            }

            let new_exponents: Vec<String> = new_exponents.iter().map(|&x| format_exponent(x)).collect();

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
        if let Some(element) = basis.elements.get_mut(el_z) {
            if element.electron_shells.is_none() {
                element.electron_shells = Some(Vec::new());
            }
            if let Some(shells) = &mut element.electron_shells {
                shells.extend(new_shells);
            }
        }
    }
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
