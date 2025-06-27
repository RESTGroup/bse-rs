use bse::prelude::*;
use std::fs::{read_to_string, write as write_file};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_basis_json() {
        let case_name = "def2-TZVPD-case-1";
        let args = BseGetBasisArgs { elements: "1-3, 49-51".to_string().into(), ..Default::default() };
        let basis = get_basis("def2-TZVP", args);
        let basis_json = serde_json::to_string_pretty(&basis).unwrap();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        write_file(format!("{manifest_dir}/tests/python_ref/{case_name}.json"), &basis_json).unwrap();
        let ref_json = read_to_string(format!("{manifest_dir}/tests/python_ref/{case_name}.json")).unwrap();
        assert_eq!(basis_json, ref_json);
    }
}
