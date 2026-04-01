use bse::prelude::*;
use std::fs::read_to_string;

#[test]
fn test_get_basis_notes_3_21_g() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ref_file = format!("{manifest_dir}/tests/python_ref/get_notes/basis_notes_3-21G.txt");

    let notes = get_basis_notes("3-21G", None);
    let ref_str = read_to_string(&ref_file).expect("Reference file not found. Run tests/python_ref/get_notes.py first.");

    // Compare line by line for better debugging
    let rust_lines: Vec<&str> = notes.lines().collect();
    let ref_lines: Vec<&str> = ref_str.lines().collect();

    for (i, (rust_line, ref_line)) in rust_lines.iter().zip(ref_lines.iter()).enumerate() {
        assert_eq!(rust_line, ref_line, "Mismatch at line {}: {:?} != {:?}", i, rust_line, ref_line);
    }

    assert_eq!(
        rust_lines.len(),
        ref_lines.len(),
        "Different number of lines: {} vs {}",
        rust_lines.len(),
        ref_lines.len()
    );
}

#[test]
fn test_get_basis_notes_def2_svp() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ref_file = format!("{manifest_dir}/tests/python_ref/get_notes/basis_notes_def2-SVP.txt");

    let notes = get_basis_notes("def2-SVP", None);
    let ref_str = read_to_string(&ref_file).expect("Reference file not found. Run tests/python_ref/get_notes.py first.");

    assert_eq!(notes, ref_str);
}

#[test]
fn test_get_family_notes_ahlrichs() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ref_file = format!("{manifest_dir}/tests/python_ref/get_notes/family_notes_ahlrichs.txt");

    let notes = get_family_notes("ahlrichs", None);
    let ref_str = read_to_string(&ref_file).expect("Reference file not found. Run tests/python_ref/get_notes.py first.");

    assert_eq!(notes, ref_str);
}

#[test]
fn test_get_family_notes_dunning() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ref_file = format!("{manifest_dir}/tests/python_ref/get_notes/family_notes_dunning.txt");

    let notes = get_family_notes("dunning", None);
    let ref_str = read_to_string(&ref_file).expect("Reference file not found. Run tests/python_ref/get_notes.py first.");

    assert_eq!(notes, ref_str);
}
