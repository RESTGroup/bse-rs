import basis_set_exchange as bse
import json
import os

assert bse.__version__ == "0.11"

# ## get_basis_fmt

out_root = "get_basis_fmt"
os.makedirs(out_root, exist_ok=True)

cfgs = [
    ("nwchem"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "nwchem"}),
    ("nwchem"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "nwchem"}),
    ("nwchem"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "nwchem"}),
    ("gaussian94"    , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "gaussian94"}),
    ("gaussian94"    , "def2-ECP"   , {"elements": "49-51"     , "fmt": "gaussian94"}),
    ("gaussian94"    , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "gaussian94"}),
    ("gaussian94lib" , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "gaussian94lib"}),
    ("gaussian94lib" , "def2-ECP"   , {"elements": "49-51"     , "fmt": "gaussian94lib"}),
    ("gaussian94lib" , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "gaussian94lib"}),
    ("psi4"          , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "psi4"}),
    ("psi4"          , "def2-ECP"   , {"elements": "49-51"     , "fmt": "psi4"}),
    ("psi4"          , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "psi4"}),
    ("molcas"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "molcas"}),
    ("molcas"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "molcas"}),
    ("molcas"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "molcas"}),
    ("molcas_library", "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "molcas_library"}),
    ("molcas_library", "def2-ECP"   , {"elements": "49-51"     , "fmt": "molcas_library"}),
    ("molcas_library", "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "molcas_library"}),
    ("qchem"         , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "qchem"}),
    ("qchem"         , "def2-ECP"   , {"elements": "49-51"     , "fmt": "qchem"}),
    ("qchem"         , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "qchem"}),
    ("gamess_us"     , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "gamess_us"}),
    ("gamess_us"     , "def2-ECP"   , {"elements": "49-51"     , "fmt": "gamess_us"}),
    ("gamess_us"     , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "gamess_us"}),
    ("orca"          , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "orca"}),
    ("orca"          , "def2-ECP"   , {"elements": "49-51"     , "fmt": "orca"}),
    ("orca"          , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "orca"}),
    ("dalton"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "dalton"}),
    ("dalton"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "dalton"}),
    ("dalton"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "dalton"}),
    ("qcschema"      , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "qcschema"}),
    ("qcschema"      , "def2-ECP"   , {"elements": "49-51"     , "fmt": "qcschema"}),
    ("qcschema"      , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "qcschema"}),
    ("cp2k"          , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "cp2k"}),
    ("cp2k"          , "def2-ECP"   , {"elements": "49-51"     , "fmt": "cp2k"}),
    ("cp2k"          , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "cp2k"}),
    ("pqs"           , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "pqs"}),
    ("pqs"           , "def2-ECP"   , {"elements": "49-51"     , "fmt": "pqs"}),
    ("pqs"           , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "pqs"}),
    ("demon2k"       , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "demon2k"}),
    ("demon2k"       , "def2-ECP"   , {"elements": "49-51"     , "fmt": "demon2k"}),
    ("demon2k"       , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "demon2k"}),
    ("turbomole"     , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "turbomole"}),
    ("turbomole"     , "def2-ECP"   , {"elements": "49-51"     , "fmt": "turbomole"}),
    ("turbomole"     , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "turbomole"}),
    ("gamess_uk"     , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "gamess_uk"}),
    ("gamess_uk"     , "def2-ECP"   , {"elements": "49-51"     , "fmt": "gamess_uk"}),
    ("gamess_uk"     , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "gamess_uk"}),
    ("molpro"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "molpro"}),
    ("molpro"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "molpro"}),
    ("molpro"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "molpro"}),
    ("cfour"         , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "cfour"}),
    ("cfour"         , "def2-ECP"   , {"elements": "49-51"     , "fmt": "cfour"}),
    ("cfour"         , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "cfour"}),
    ("acesii"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "acesii"}),
    ("acesii"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "acesii"}),
    ("acesii"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "acesii"}),
    ("bdf"           , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "bdf"}),
    ("bdf"           , "def2-ECP"   , {"elements": "49-51"     , "fmt": "bdf"}),
    ("bdf"           , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "bdf"}),
    ("fhiaims"       , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "fhiaims"}),
    ("jaguar"        , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "jaguar"}),
    ("jaguar"        , "def2-ECP"   , {"elements": "49-51"     , "fmt": "jaguar"}),
    ("jaguar"        , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "jaguar"}),
    ("crystal"       , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "crystal"}),
    ("crystal"       , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "crystal"}),
    ("veloxchem"     , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "veloxchem"}),
]

# # %%time
for (scene, basis, kwargs) in cfgs:
    with open(f"{out_root}/{basis}-{scene}.txt", "w") as f:
        token = bse.get_basis(basis, **kwargs, header=False)
        f.write(token)


