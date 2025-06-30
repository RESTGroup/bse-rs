//! One-center integrals for Gaussian-type and Slater-type orbitals
//!
//! Written by Susi Lehtola, 2020 (BSE original Python code)
//!
//! Modified for rust in this project.

use crate::misc;
use libm::{sqrt, tgamma};

fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    assert!(a[0].len() == b.len(), "Matrix dimensions do not match for multiplication.");

    let m = a.len();
    let n = b[0].len();
    let p = a[0].len();

    let mut result = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {
            for k in 0..p {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

fn transform(c: &[Vec<f64>], p: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let ct = misc::transpose_matrix(c);
    matmul(c, &matmul(p, &ct))
}

fn vec_to_float(m: &[String]) -> Vec<f64> {
    m.iter().map(|s| s.parse::<f64>().unwrap()).collect()
}

fn mat_to_float(m: &[Vec<String>]) -> Vec<Vec<f64>> {
    m.iter().map(|row| vec_to_float(row)).collect()
}

fn vec_to_int(m: &[String]) -> Vec<i32> {
    m.iter().map(|s| s.parse::<i32>().unwrap()).collect()
}

fn zero_matrix(n: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; n]; n]
}

/// Returns a normalized contraction matrix given the overlap matrix.
fn normalize_contraction(contr: &[Vec<f64>], ovl: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // Number of contractions
    let nprim = contr[0].len();
    // Number of primitives
    let ncontr = contr.len();

    // Size check: we want the primitive overlap
    assert!(ovl.len() == nprim, "Overlap matrix dimensions do not match");
    assert!(ovl[0].len() == nprim, "Overlap matrix dimensions do not match");

    // Transform the overlap to the contracted basis
    let ovl = transform(contr, ovl);
    let normfac: Vec<f64> = (0..ncontr).map(|i| 1.0 / sqrt(ovl[i][i])).collect();

    // Form the normalized contraction matrix
    let mut norm_contr = contr.to_vec();
    for icontr in 0..ncontr {
        for iprim in 0..nprim {
            norm_contr[icontr][iprim] *= normfac[icontr];
        }
    }
    norm_contr
}

/// Computes the primitive overlap matrix for the given exponents, assuming the
/// basis functions are of the spherical form $r^l exp(-z r^2)$
fn gto_overlap(exps: &[f64], l: i32) -> Vec<Vec<f64>> {
    assert!(l >= 0, "Angular momentum l must be non-negative");

    fn ovl(a: f64, b: f64, l: i32) -> f64 {
        let ab = 0.5 * (a + b);
        ((a * b) / (ab * ab)).powf((l as f64) / 2.0 + 3.0 / 4.0)
    }

    // Initialize memory
    let n = exps.len();
    let mut overlaps = zero_matrix(n);

    // Compute overlaps
    for i in 0..n {
        for j in 0..=i {
            overlaps[i][j] = ovl(exps[i], exps[j], l);
            overlaps[j][i] = overlaps[i][j];
        }
    }
    overlaps
}

/// Computes the overlap matrix in the contracted basis, assuming the basis
/// functions are of the spherical form $r^l \sum_i c_i exp(-z_i r^2)$.
pub fn gto_overlap_contr(exps0: &[String], contr0: &[Vec<String>], l: i32) -> Vec<Vec<f64>> {
    // Convert exponents and contractions to floating point
    let exps = vec_to_float(exps0);
    let contr = mat_to_float(contr0);

    // Get primitive integrals
    let ovl = gto_overlap(&exps, l);
    transform(&contr, &ovl)
}

/// Computes the <r> matrix for the given exponents, assuming the basis
/// functions are of the normalized spherical form $r^l exp(-z r^2)$.
fn gto_R(exps: &[f64], l: i32) -> Vec<Vec<f64>> {
    assert!(l >= 0, "Angular momentum l must be non-negative");

    fn rval(a: f64, b: f64, l: i32) -> f64 {
        let ab = 0.5 * (a + b);
        let sqrtab = sqrt(a * b);
        1.0 / sqrt(sqrtab) * (sqrtab / ab).powi(l + 2)
    }

    // Initialize memory
    let n = exps.len();
    let mut rmat = zero_matrix(n);
    let prefactor = tgamma(l as f64 + 2.0) / (sqrt(2.0) * tgamma(l as f64 + 1.5));

    // Compute matrix elements
    for i in 0..n {
        for j in 0..=i {
            rmat[i][j] = prefactor * rval(exps[i], exps[j], l);
            rmat[j][i] = rmat[i][j];
        }
    }
    rmat
}

/// Computes the r matrix in the contracted basis, assuming the basis functions
/// are of the spherical form $r^l \sum_i c_i exp(-z_i r^2)$. The function also
/// takes care of proper normalization.
pub fn gto_R_contr(exps0: &[String], contr0: &[Vec<String>], l: i32) -> Vec<Vec<f64>> {
    // Convert exponents and contractions to floating point
    let exps = vec_to_float(exps0);
    let contr = mat_to_float(contr0);

    // Get primitive integrals
    let rmat = gto_R(&exps, l);
    let ovl = gto_overlap(&exps, l);

    // Normalize the contraction
    let norm_contr = normalize_contraction(&contr, &ovl);
    // Transform to normalized contracted form
    transform(&norm_contr, &rmat)
}

/// Computes the $r^2$ matrix in the contracted basis, assuming the basis
/// functions are of the spherical form $r^l \sum_i c_i exp(-z_i r^2)$. The
/// function also takes care of proper normalization.
fn gto_Rsq(exps: &[f64], l: i32) -> Vec<Vec<f64>> {
    assert!(l >= 0, "Angular momentum l must be non-negative");

    fn rsq(a: f64, b: f64, l: i32) -> f64 {
        let ab = 0.5 * (a + b);
        ((a * b) / (ab * ab)).powf(l as f64 / 2.0 + 3.0 / 4.0) / ab
    }

    // Initialize memory
    let n = exps.len();
    let mut rsqs = zero_matrix(n);
    let prefactor = 3.0 / 4.0 + l as f64 / 2.0;

    // Compute matrix elements
    for i in 0..n {
        for j in 0..=i {
            rsqs[i][j] = prefactor * rsq(exps[i], exps[j], l);
            rsqs[j][i] = rsqs[i][j];
        }
    }
    rsqs
}

/// Computes the r^2 matrix in the contracted basis, assuming the basis
/// functions are of the spherical form $r^l \sum_i c_i exp(-z_i r^2)$. The
/// function also takes care of proper normalization.
pub fn gto_Rsq_contr(exps0: &[String], contr0: &[Vec<String>], l: i32) -> Vec<Vec<f64>> {
    // Convert exponents and contractions to floating point
    let exps = vec_to_float(exps0);
    let contr = mat_to_float(contr0);

    // Get primitive integrals
    let rsqs = gto_Rsq(&exps, l);
    let ovl = gto_overlap(&exps, l);

    // Normalize the contraction
    let norm_contr = normalize_contraction(&contr, &ovl);
    // Transform to normalized contracted form
    transform(&norm_contr, &rsqs)
}

/// Computes the primitive overlap matrix for the given exponents and primary
/// quantum numbers, assuming the basis functions are of the spherical form
/// $r^(n-1) exp(-z r)$.
fn sto_overlap(exps: &[f64], ns: &[i32]) -> Vec<Vec<f64>> {
    assert_eq!(exps.len(), ns.len(), "exps and ns must have the same length");

    fn ovl(za: f64, zb: f64, na: i32, nb: i32) -> f64 {
        let numerator = tgamma((na + nb + 1) as f64);
        let denominator = sqrt(tgamma((2 * na + 1) as f64) * tgamma((2 * nb + 1) as f64));
        let prefactor = numerator / denominator;
        let za_part = za.powf(na as f64 + 0.5);
        let zb_part = zb.powf(nb as f64 + 0.5);
        let denom_part = (0.5 * (za + zb)).powf((na + nb + 1) as f64);

        prefactor * za_part * zb_part / denom_part
    }

    // Initialize memory
    let n = exps.len();
    let mut overlaps = zero_matrix(n);

    // Compute overlaps
    for i in 0..n {
        for j in 0..=i {
            overlaps[i][j] = ovl(exps[i], exps[j], ns[i], ns[j]);
            overlaps[j][i] = overlaps[i][j];
        }
    }
    overlaps
}

/// Computes the overlap matrix in the contracted basis, assuming the basis
/// functions are of the spherical form $r^(n-1) exp(-z r)$.
pub fn sto_overlap_contr(exps0: &[String], contr0: &[Vec<String>], ns0: &[String]) -> Vec<Vec<f64>> {
    // Convert exponents, contractions, and quantum numbers to floating point
    let exps = vec_to_float(exps0);
    let contr = mat_to_float(contr0);
    let ns = vec_to_int(ns0);

    // Get primitive integrals
    let ovl = sto_overlap(&exps, &ns);
    transform(&contr, &ovl)
}

/// Computes the primitive $r^2$ matrix for the given exponents and primary
/// quantum numbers, assuming the basis functions are of the spherical form
/// $r^(n-1) exp(-z r)$.
fn sto_Rsq(exps: &[f64], ns: &[i32]) -> Vec<Vec<f64>> {
    assert_eq!(exps.len(), ns.len(), "exps and ns must have the same length");

    fn rsq(za: f64, zb: f64, na: i32, nb: i32) -> f64 {
        let numerator = tgamma((na + nb + 3) as f64);
        let denominator = sqrt(tgamma((2 * na + 1) as f64) * tgamma((2 * nb + 1) as f64));
        let prefactor = numerator / denominator;
        let za_part = za.powf(na as f64 + 0.5);
        let zb_part = zb.powf(nb as f64 + 0.5);
        let denom_part = (0.5 * (za + zb)).powf((na + nb + 3) as f64);

        prefactor * za_part * zb_part / denom_part
    }

    // Initialize memory
    let n = exps.len();
    let mut rsqs = zero_matrix(n);

    // Compute matrix elements
    for i in 0..n {
        for j in 0..=i {
            rsqs[i][j] = rsq(exps[i], exps[j], ns[i], ns[j]);
            rsqs[j][i] = rsqs[i][j];
        }
    }
    rsqs
}

/// Computes the r^2 matrix in the contracted basis, assuming the basis
/// functions are of the spherical form $r^(n-1) exp(-z r)$.
pub fn sto_Rsq_contr(exps0: &[String], contr0: &[Vec<String>], ns0: &[String]) -> Vec<Vec<f64>> {
    // Convert exponents, contractions, and quantum numbers to floating point
    let exps = vec_to_float(exps0);
    let contr = mat_to_float(contr0);
    let ns = vec_to_int(ns0);

    // Get primitive integrals
    let rsqs = sto_Rsq(&exps, &ns);
    let ovl = sto_overlap(&exps, &ns);

    // Normalize the contraction
    let norm_contr = normalize_contraction(&contr, &ovl);
    // Transform to normalized contracted form
    transform(&norm_contr, &rsqs)
}
