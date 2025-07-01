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
]

# # %%time
for (scene, basis, kwargs) in cfgs:
    with open(f"{out_root}/{basis}-{scene}.txt", "w") as f:
        token = bse.get_basis(basis, **kwargs, header=False)
        f.write(token)


