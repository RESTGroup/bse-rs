use bse::prelude::*;
use rstest::rstest;
use std::fs::{read_to_string, write as write_from_string};

#[cfg(test)]
mod test {
    use super::*;

    #[rstest]
    #[case("cc-pVTZ", "1, 6-O")]
    #[case("def2-TZVPD", "1-3, 49-51")]
    fn test_get_basis_json(#[case] basis: &str, #[case] elements: &str) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let ref_file = format!("{manifest_dir}/tests/python_ref/get_basis_json/{basis}.json");
        let do_write = true;

        let args = BseGetBasisArgs { elements: elements.to_string().into(), ..Default::default() };
        let basis = get_basis(basis, args);
        let basis_json = serde_json::to_string_pretty(&basis).unwrap();

        if do_write {
            let write_file = format!("{manifest_dir}/tests/tmp.json");
            write_from_string(&write_file, &basis_json).unwrap();
        }

        let ref_json = read_to_string(ref_file).unwrap();
        assert_eq!(basis_json, ref_json);
    }
}
