import basis_set_exchange as bse
import json
import os

assert bse.__version__ == "0.12"

# ## get_truhlar_calendarize

out_root = "get_truhlar_calendarize"
os.makedirs(out_root, exist_ok=True)

# Test matrix:
# - aug-cc-pVTZ -> jun/apr
# - aug-cc-pVQZ -> jun/apr
# - elements: H, C, Ga, Zn
cfgs = [
    # aug-cc-pVTZ tests
    ("aug-cc-pVTZ", "jun", {"elements": "1, 6, 31, 30"}),  # H, C, Ga, Zn
    ("aug-cc-pVTZ", "apr", {"elements": "1, 6, 31, 30"}),  # H, C, Ga, Zn
    # aug-cc-pVQZ tests
    ("aug-cc-pVQZ", "jun", {"elements": "1, 6, 31, 30"}),  # H, C, Ga, Zn
    ("aug-cc-pVQZ", "apr", {"elements": "1, 6, 31, 30"}),  # H, C, Ga, Zn
]

for (basis, month, kwargs) in cfgs:
    # Get the aug basis first
    aug_basis = bse.get_basis(basis, **kwargs)

    # Apply truhlar_calendarize
    result = bse.manip.truhlar_calendarize(aug_basis, month)

    # Clean up for comparison
    if "data_source" in result:
        del result["data_source"]

    # Create clean element string for filename (H-C-Ga-Zn format)
    el_str = kwargs["elements"].replace(" ", "").replace(",", "-")

    # Write JSON output
    filename = f"{basis}-{month}-{el_str}.json"
    with open(f"{out_root}/{filename}", "w") as f:
        json.dump(result, f, indent=2)

    # Write Gaussian94 format for additional comparison
    fmt_output = bse.writers.write_formatted_basis_str(result, "gaussian94")
    filename_g94 = f"{basis}-{month}-{el_str}.g94"
    with open(f"{out_root}/{filename_g94}", "w") as f:
        f.write(fmt_output)

print("Generated truhlar_calendarize reference files")