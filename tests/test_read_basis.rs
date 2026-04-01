use assert_json_diff::assert_json_include;
use bse::prelude::*;
use rstest::rstest;
use std::fs::{read_to_string, write as write_from_string};

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case("nwchem"        , "nwchem"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("nwchem"        , "nwchem"        , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("nwchem"        , "nwchem"        , "def2-TZVP" , ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("gaussian94"    , "gaussian94"    , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("gaussian94"    , "gaussian94"    , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("gaussian94"    , "gaussian94"    , "def2-TZVP" , ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("turbomole"     , "turbomole"     , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("turbomole"     , "turbomole"     , "def2-TZVP" , ["elements = '1-3, 49-51'"].join("\n"))]
    #[case("dalton"        , "dalton"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("molcas"        , "molcas"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("molcas"        , "molcas"        , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("molpro"        , "molpro"        , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("molpro"        , "molpro"        , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("cfour"         , "cfour"         , "cc-pVDZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("cfour"         , "cfour"         , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("gamess_us"      , "gamess_us"      , "cc-pVDZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("gamess_us"      , "gamess_us"      , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("cp2k"          , "cp2k"           , "cc-pVTZ"   , ["elements = '1, 6-O'"    ].join("\n"))]
    #[case("cp2k"          , "cp2k"           , "def2-ECP"  , ["elements = '49-51'"     ].join("\n"))]
    #[case("crystal"       , "crystal"        , "def2-SVP"  , ["elements = '1, 6-O'"    ].join("\n"))]
    fn test_get_basis(#[case] scene: &str, #[case] dump_fmt: &str, #[case] basis: &str, #[case] args: String) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let ref_file = format!("{manifest_dir}/tests/python_ref/read_basis_fmt/{basis}-{scene}.json");
        let write_file = format!("{manifest_dir}/tests/tmp.json");

        let args = toml::from_str::<BseGetBasisArgs>(&args).unwrap();
        let basis_str = get_formatted_basis(basis, dump_fmt, args);
        let basis = read_formatted_basis_str(&basis_str, scene);

        let basis_str = serde_json::to_string_pretty(&basis).unwrap();
        let basis_json = serde_json::from_str::<serde_json::Value>(&basis_str).unwrap();
        write_from_string(&write_file, &basis_str).unwrap();

        // Check if reference file exists
        if std::path::Path::new(&ref_file).exists() {
            let ref_str = read_to_string(ref_file).unwrap();
            let ref_json = serde_json::from_str::<serde_json::Value>(&ref_str).unwrap();
            assert_json_include!(actual: basis_json, expected: ref_json);
        }
    }
}
