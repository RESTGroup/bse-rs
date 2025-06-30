//! Sorting of BSE related dictionaries and data.

use crate::prelude::*;

/// Computes the spatial extent for the orbitals on the shell.
pub fn spatial_extent(sh: &BseElectronShell) -> Vec<f64> {
    let mut rsq = Vec::new();

    if sh.function_type.starts_with("gto") {
        // Catches GTO, spherical and cartesian
        if sh.angular_momentum.len() == 1 {
            // General contraction
            let rsq_mat = ints::gto_Rsq_contr(&sh.exponents, &sh.coefficients, sh.angular_momentum[0]);
            rsq = (0..rsq_mat.len()).map(|i| rsq_mat[i][i]).collect();
        } else {
            // SP shell etc
            for (iam, &am) in sh.angular_momentum.iter().enumerate() {
                let coefficients = vec![sh.coefficients[iam].clone()];
                let rsq_mat = ints::gto_Rsq_contr(&sh.exponents, &coefficients, am);
                // We should only have a single element
                assert!(rsq_mat.len() == 1 && rsq_mat[0].len() == 1);
                rsq.push(rsq_mat[0][0]);
            }
        }
    } else {
        panic!("Function type {} not handled", sh.function_type);
    }

    rsq
}

/// Sort a basis set shell into a standard order.
pub fn sort_shell(shell: &mut BseElectronShell) {
    let tmp_c = shell.coefficients.clone();
    let tmp_z = shell.exponents.clone();

    // Exponents should be in decreasing order
    let mut zidx: Vec<usize> = (0..tmp_z.len()).collect();
    zidx.sort_by(|&a, &b| {
        let za = tmp_z[a].parse::<f64>().unwrap();
        let zb = tmp_z[b].parse::<f64>().unwrap();
        zb.partial_cmp(&za).unwrap()
    });

    let cidx: Vec<usize> = if shell.angular_momentum.len() == 1 {
        let rsq_vec = spatial_extent(shell);
        let mut indices: Vec<usize> = (0..rsq_vec.len()).collect();
        indices.sort_by(|&a, &b| rsq_vec[a].partial_cmp(&rsq_vec[b]).unwrap());
        indices
    } else {
        // This is an SP shell etc; we only have one contraction per am
        // so we don't have to sort the contractions.
        (0..tmp_c.len()).collect()
    };

    // Collect the exponents and coefficients
    let newexp: Vec<String> = zidx.iter().map(|&i| tmp_z[i].clone()).collect();
    let newcoef: Vec<Vec<String>> = cidx.iter().map(|&i| zidx.iter().map(|&j| tmp_c[i][j].clone()).collect()).collect();

    shell.exponents = newexp;
    shell.coefficients = newcoef;
}

/// Sort a list of basis set shells into a standard order
///
/// The order within a shell is by decreasing value of the exponent.
///
/// The order of the shell list is in increasing angular momentum, and then
/// by decreasing number of primitives, then decreasing value of the largest
/// exponent.
pub fn sort_shells(shells: &mut Vec<BseElectronShell>) {
    // Sort primitives within each shell
    for shell in shells.iter_mut() {
        sort_shell(shell);
    }

    // Calculate minimum spatial extent for each shell
    let mut shells_with_extents: Vec<(BseElectronShell, f64)> = shells
        .drain(..)
        .map(|shell| {
            let min_extent = spatial_extent(&shell).into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            (shell, min_extent)
        })
        .collect();

    // Sort by increasing AM and then by increasing spatial extent
    shells_with_extents.sort_by(|a, b| {
        let am_a = a.0.angular_momentum.iter().max().unwrap();
        let am_b = b.0.angular_momentum.iter().max().unwrap();

        am_a.cmp(am_b).then(a.1.partial_cmp(&b.1).unwrap())
    });

    // Extract the sorted shells
    *shells = shells_with_extents.into_iter().map(|(shell, _)| shell).collect();
}

/// Sort a list of ECP potentials into a standard order.
///
/// The order within a potential is not modified.
///
/// The order of the shell list is in increasing angular momentum, with the
/// largest angular momentum being moved to the front.
pub fn sort_potentials(potentials: &mut Vec<BseEcpPotential>) {
    // Sort by increasing angular momentum (comparing the first element of the
    // vector)
    potentials.sort_by(|a, b| {
        let am_a = a.angular_momentum.first().unwrap_or(&0);
        let am_b = b.angular_momentum.first().unwrap_or(&0);
        am_a.cmp(am_b)
    });

    // Move the last element (now with largest AM) to the front
    if !potentials.is_empty() {
        let last = potentials.pop().unwrap();
        potentials.insert(0, last);
    }
}

/// Sorts all the information in a basis set into a standard order.
pub fn sort_basis(basis: &mut BseBasis) {
    // Sort electron shells and ECP potentials for each element
    for (_, element) in basis.elements.iter_mut() {
        if let Some(ref mut electron_shells) = element.electron_shells {
            sort_shells(electron_shells);
        }

        if let Some(ref mut ecp_potentials) = element.ecp_potentials {
            sort_potentials(ecp_potentials);
        }
    }
}
