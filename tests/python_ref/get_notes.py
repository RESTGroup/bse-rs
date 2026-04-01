import basis_set_exchange as bse
import os

out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'get_notes')
os.makedirs(out_dir, exist_ok=True)

# Test family notes
notes = bse.get_family_notes('ahlrichs')
with open(f'{out_dir}/family_notes_ahlrichs.txt', 'w') as f:
    f.write(notes)

notes = bse.get_family_notes('dunning')
with open(f'{out_dir}/family_notes_dunning.txt', 'w') as f:
    f.write(notes)

# Test basis notes
notes = bse.get_basis_notes('3-21G')
with open(f'{out_dir}/basis_notes_3-21G.txt', 'w') as f:
    f.write(notes)

notes = bse.get_basis_notes('def2-SVP')
with open(f'{out_dir}/basis_notes_def2-SVP.txt', 'w') as f:
    f.write(notes)

print("Reference files generated successfully")