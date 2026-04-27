# Changelog

## v0.1.4 -- 2026-04-27

Enhancement:

- `None` can be an argument passed to `get_basis`.

Behavior change:

- Optimize BSE data directory detection to avoid spawning Python process. This will check python executable path, and try to find BSE data directory relative to the python executable.

## v0.1.3 -- 2026-04-23

Bug fix:

- Fix a bug in nwchem handling primitive matrix check.

## v0.1.2 -- 2026-04-22

API functionality addition:

- Added manipulation functions (manip.rs), such as `prune_basis_in_element`, `uncontract_spdf_in_element`. These functions mainly focus on manipulation to `BseBasisElement`, instead to the whole basis set.

## v0.1.1 -- 2026-04-20

Cargo dependency fixes. Now feature `remote` will require dependency of ureq, and we removes cargo feature of ureq.

## v0.1.0 -- 2026-04-02

Initial bse-rs release.

This should already contains virtually all the features in the Python version.
