import basis_set_exchange as bse
import json
import os

assert bse.__version__ == "0.11"

# ## get_basis_fmt

out_root = "get_basis_fmt"
os.makedirs(out_root, exist_ok=True)

cfgs = [
    ("nwchem"    , "cc-pVTZ"    , {"elements": "1, 6-O"    , "fmt": "nwchem"}),
    ("nwchem"    , "def2-TZVPD" , {"elements": "1-3, 49-51", "fmt": "nwchem"}),
]

for (scene, basis, kwargs) in cfgs:
    with open(f"{out_root}/{basis}-{scene}.txt", "w") as f:
        token = bse.get_basis(basis, **kwargs, header=False)
        f.write(token)


