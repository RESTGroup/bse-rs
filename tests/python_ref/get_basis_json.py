import basis_set_exchange as bse
import json
import os

# ## get_basis_json

out_root = "get_basis_json"
os.makedirs(out_root, exist_ok=True)

# ### naive

cfgs = [
    ("cc-pVTZ", "1, 6-O"),
    ("def2-TZVPD", "1-3, 49-51"),
]

for (basis, elements) in cfgs:
    with open(f"{out_root}/{basis}.json", "w") as f:
        json.dump(bse.get_basis(basis, elements=elements), f, indent=2)








