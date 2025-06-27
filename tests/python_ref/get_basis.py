import basis_set_exchange as bse
import json
import os

# ## get_basis

# ### 

with open("def2-TZVPD-case-1.json", "w") as f:
    json.dump(bse.get_basis("def2-TZVPD", elements="1-3, 49-51"), f, indent=2)



data_dir = os.environ.get("BSE_DATA_DIR")





table_relpath = "def2-TZVP.1.table.json"

data_dir

data_dir = "/home/a/Git-Others/basis_set_exchange/basis_set_exchange/data"
table_relpath = "def2-TZVP.1.table.json"


