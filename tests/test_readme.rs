pub use bse::prelude::*;

#[test]
fn test_readme() {
    use bse::prelude::*;

    // Get basis set as structured object
    let args = BseGetBasisArgsBuilder::default().elements("H, C-O".to_string()).build().unwrap();
    let basis = get_basis("cc-pVTZ", args);
    println!("Basis: {} ({})", basis.name, basis.family);

    // use TOML configuration for arguments
    let args_string = r#"
        elements = "H, C-O"
        augment_diffuse = 1
    "#;
    let basis = get_basis("cc-pVTZ", args_string);
    println!("Basis: {} ({})", basis.name, basis.family);

    // Get formatted output for quantum chemistry software
    let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).header(true).build().unwrap();
    let output = get_formatted_basis("sto-3g", "nwchem", args);
    println!("{}", output);

    // Read basis from file
    // let content = std::fs::read_to_string("basis.nw").unwrap();
    // this file is not included in the repo, so create one temporarily here:
    let content = write_formatted_basis_str(&basis, "nw", None);
    // write to basis.nw
    std::fs::write("basis.nw", &content).unwrap();
    let content = std::fs::read_to_string("basis.nw").unwrap();
    let basis_minimal = read_formatted_basis_str(&content, "nwchem");
    println!("Basis: {:?}", basis_minimal);
    // remove the temporary file
    std::fs::remove_file("basis.nw").unwrap();

    // Apply manipulations
    let args = BseGetBasisArgsBuilder::default().uncontract_general(true).augment_diffuse(2).build().unwrap();
    let basis = get_basis("def2-SVP", args);
    println!("Basis: {} ({})", basis.name, basis.family);

    // Get Truhlar calendar basis directly (seamless integration)
    let basis = get_basis("jul-cc-pVTZ", BseGetBasisArgs::default());
    println!("Basis: {} ({})", basis.name, basis.family);
    let basis = get_basis("maug-cc-pVDZ", BseGetBasisArgs::default()); // Auto-selects jun for DZ
    println!("Basis: {} ({})", basis.name, basis.family);
}
