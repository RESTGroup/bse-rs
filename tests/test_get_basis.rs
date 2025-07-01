use assert_json_diff::assert_json_include;
use bse::prelude::*;
use rstest::rstest;
use std::fs::{read_to_string, write as write_from_string};

#[cfg(test)]
mod test {
    use super::*;

    #[rstest]
    #[case("naive"                 , "cc-pVTZ"    , ["elements = '1, 6-O'"         ,                                ].join("\n"))]
    #[case("naive"                 , "def2-TZVPD" , ["elements = '1-3, 49-51'"     ,                                ].join("\n"))]
    #[case("remove_free_primitives", "cc-pVTZ"    , ["elements = '1, 6-O'"         , "remove_free_primitives = true"].join("\n"))]
    #[case("remove_free_primitives", "def2-TZVPD" , ["elements = '1-3, 49-51'"     , "remove_free_primitives = true"].join("\n"))]
    #[case("make_general"          , "aug-cc-pVTZ", ["elements = '1, 6-O'"         , "make_general = true"          ].join("\n"))]
    #[case("optimize_general"      , "aug-cc-pVTZ", ["elements = '1, 6-O'"         , "optimize_general = true"      ].join("\n"))]
    #[case("uncontract_segmented"  , "aug-cc-pVTZ", ["elements = '1, 6-O'"         , "uncontract_segmented = true"  ].join("\n"))]
    #[case("uncontract_general"    , "aug-cc-pVTZ", ["elements = '1, 6-O'"         , "uncontract_general = true"    ].join("\n"))]
    #[case("uncontract_spdf"       , "6-31G"      , ["elements = '1, 6-O'"         , "uncontract_spdf = true"       ].join("\n"))]
    #[case("augment_diffuse"       , "cc-pVTZ"    , ["elements = '1, 6-O'"         , "augment_diffuse = 2"          ].join("\n"))]
    #[case("augment_steep"         , "cc-pVTZ"    , ["elements = '1, 6-O'"         , "augment_steep = 2"            ].join("\n"))]
    #[case("get_aux"               , "def2-SVP"   , ["elements = '1,6,15,25,59,86'", "get_aux = 1"                  ].join("\n"))]
    #[case("get_aux"               , "cc-pVTZ"    , ["elements = '1,6,15,25'"      , "get_aux = 1"                  ].join("\n"))]
    #[case("get_abs"               , "def2-SVP"   , ["elements = '1,6,15,25,59,86'", "get_aux = 2"                  ].join("\n"))]
    #[case("get_abs"               , "cc-pVTZ"    , ["elements = '1,6,15,25'"      , "get_aux = 2"                  ].join("\n"))]
    fn test_get_basis(#[case] scene: &str, #[case] basis: &str, #[case] args: String) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let ref_file = format!("{manifest_dir}/tests/python_ref/get_basis_json/{basis}-{scene}.json");
        let write_file = format!("{manifest_dir}/tests/tmp.json");

        let args = toml::from_str::<BseGetBasisArgs>(&args).unwrap();
        let basis = get_basis(basis, args);

        let basis_str = serde_json::to_string_pretty(&basis).unwrap();
        let basis_json = serde_json::from_str::<serde_json::Value>(&basis_str).unwrap();
        write_from_string(&write_file, &basis_str).unwrap();

        let ref_str = read_to_string(ref_file).unwrap();
        let ref_json = serde_json::from_str::<serde_json::Value>(&ref_str).unwrap();
        assert_json_include!(actual: basis_json, expected: ref_json);
    }

    #[rstest]
    #[case("nwchem"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("nwchem"        , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("nwchem"        , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("gaussian94"    , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("gaussian94"    , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("gaussian94"    , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("gaussian94lib" , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("gaussian94lib" , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("gaussian94lib" , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("psi4"          , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("psi4"          , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("psi4"          , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("molcas"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("molcas"        , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("molcas"        , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("qchem"         , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("qchem"         , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("qchem"         , "def2-TZVPD", ["elements = '1-3, 49-51'"].join("\n"))]
    fn test_get_formatted_basis(#[case] fmt: &str, #[case] basis: &str, #[case] args: String) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let ref_file = format!("{manifest_dir}/tests/python_ref/get_basis_fmt/{basis}-{fmt}.txt");
        let write_file = format!("{manifest_dir}/tests/tmp.txt");

        let mut args = toml::from_str::<BseGetBasisArgs>(&args).unwrap();
        args.header = false;
        let basis_str = get_formatted_basis(basis, fmt, args);
        write_from_string(&write_file, &basis_str).unwrap();

        let ref_str = read_to_string(ref_file).unwrap();

        use similar::{ChangeTag, TextDiff};
        let diff = TextDiff::from_lines(&basis_str, &ref_str);
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            print!("{sign}{change}");
        }

        assert_eq!(basis_str, ref_str);
    }
}
