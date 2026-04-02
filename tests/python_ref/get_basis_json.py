import basis_set_exchange as bse
import json
import os

assert bse.__version__ == "0.12"

# ## get_basis_json

out_root = "get_basis_json"
os.makedirs(out_root, exist_ok=True)

cfgs = [
    ("naive"                 , "cc-pVTZ"    , {"elements": "1, 6-O"         ,}),
    ("naive"                 , "def2-TZVPD" , {"elements": "1-3, 49-51"     ,}),
    ("remove_free_primitives", "cc-pVTZ"    , {"elements": "1, 6-O"         , "remove_free_primitives": True}),
    ("remove_free_primitives", "def2-TZVPD" , {"elements": "1-3, 49-51"     , "remove_free_primitives": True}),
    ("make_general"          , "aug-cc-pVTZ", {"elements": "1, 6-O"         , "make_general": True}),
    ("optimize_general"      , "aug-cc-pVTZ", {"elements": "1, 6-O"         , "optimize_general": True}),
    ("uncontract_segmented"  , "aug-cc-pVTZ", {"elements": "1, 6-O"         , "uncontract_segmented": True}),
    ("uncontract_general"    , "aug-cc-pVTZ", {"elements": "1, 6-O"         , "uncontract_general": True}),
    ("uncontract_spdf"       , "6-31G"      , {"elements": "1, 6-O"         , "uncontract_spdf": True}),
    ("augment_diffuse"       , "cc-pVTZ"    , {"elements": "1, 6-O"         , "augment_diffuse": 2}),
    ("augment_steep"         , "cc-pVTZ"    , {"elements": "1, 6-O"         , "augment_steep": 2}),
    ("get_aux"               , "def2-SVP"   , {"elements": "1,6,15,25,59,86", "get_aux": 1}),
    ("get_aux"               , "cc-pVTZ"    , {"elements": "1,6,15,25"      , "get_aux": 1}),
    ("get_abs"               , "def2-SVP"   , {"elements": "1,6,15,25,59,86", "get_aux": 2}),
    ("get_abs"               , "cc-pVTZ"    , {"elements": "1,6,15,25"      , "get_aux": 2}),
]

for (scene, basis, kwargs) in cfgs:
    with open(f"{out_root}/{basis}-{scene}.json", "w") as f:
        dct = bse.get_basis(basis, **kwargs)
        if "data_source" in dct:
            del dct["data_source"]
        json.dump(dct, f, indent=2)


