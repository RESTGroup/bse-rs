# Implementation Plan: Missing Features from Python BSE

This document tracks the missing features identified by comparing the Rust `bse-rs` implementation with the Python `basis_set_exchange` library.

**Python Reference**: `/home/a/Git-Others/basis_set_exchange`

---

## Priority 1: Core API Functions (Metadata & Queries)

These are essential for discoverability and querying available basis sets.

| Function | Description | Status | File |
|----------|-------------|--------|------|
| `get_all_basis_names` | List all basis set names | ✅ DONE | `src/api.rs` |
| `get_families` | List all basis set families | ✅ DONE | `src/api.rs` |
| `filter_basis_sets` | Filter by substr, family, role, elements | ✅ DONE | `src/api.rs` |
| `get_roles` | Return available roles (jfit, jkfit, rifit, etc.) | ✅ DONE | `src/api.rs` |
| `lookup_basis_by_role` | Lookup auxiliary basis by role | ✅ DONE | `src/api.rs` |
| `get_formats` | Return available writer formats | ✅ DONE | `src/api.rs` |
| `get_reader_formats` | Return available reader formats | ✅ DONE | `src/readers/read.rs` |
| `get_writer_formats` | Return available writer formats | ✅ DONE | `src/writers/write.rs` |

---

## Priority 2: Reference/Citation Handling

Reference data is important for citing basis sets in publications.

| Function | Description | Status | File |
|----------|-------------|--------|------|
| `get_reference_data` | Get all reference data from REFERENCES.json | ✅ DONE | `src/api.rs` |
| `get_references` | Get citations for a specific basis set | ✅ DONE | `src/api.rs` |
| `get_reference_formats` | Return available reference formats | ✅ DONE | `src/refconverters/convert.rs` |
| `convert_references` | Convert references to bib/txt/json | ✅ DONE | `src/refconverters/convert.rs` |

**Modules created**: `src/refconverters/` (bib, ris, endnote, txt, json converters), `src/references.rs` (compact_references, reference_text)

---

## Priority 3: Notes Handling

Notes provide important context about basis sets and families.

| Function | Description | Status | File |
|----------|-------------|--------|------|
| `get_basis_notes` | Get notes for a specific basis set | ❌ TODO | `src/api.rs` or new `notes.rs` |
| `get_family_notes` | Get notes for a basis set family | ❌ TODO | `src/api.rs` or new `notes.rs` |
| `has_basis_notes` | Check if basis notes exist | ❌ TODO | `src/api.rs` |
| `has_family_notes` | Check if family notes exist | ❌ TODO | `src/api.rs` |

**New module needed**: `src/notes.rs` for notes processing with reference substitution.

---

## Priority 4: Missing Readers

| Format | Description | Status | File |
|--------|-------------|--------|------|
| `molcas` | Molcas format | ✅ DONE | `src/readers/molcas.rs` |
| `molpro` | Molpro format | ✅ DONE | `src/readers/molpro.rs` |
| `libmol` | Molpro system library | ❌ TODO | `src/readers/libmol.rs` |
| `genbas` / `cfour` | CFOUR/ACES2 format | ✅ DONE | `src/readers/genbas.rs` |
| `gbasis` | GBasis format | ❌ TODO | `src/readers/gbasis.rs` |
| `gamess_us` | GAMESS US format | ✅ DONE | `src/readers/gamess_us.rs` |
| `demon2k` | deMon2k format | ❌ TODO | `src/readers/demon2k.rs` |
| `cp2k` | CP2K format | ✅ DONE | `src/readers/cp2k.rs` |
| `crystal` | Crystal format | ✅ DONE | `src/readers/crystal.rs` |
| `veloxchem` | VeloxChem format | ❌ TODO | `src/readers/veloxchem.rs` |
| `ricdlib` | MolCAS RICDlib format | ❌ TODO | `src/readers/ricdlib.rs` |
| `json` | JSON/BSE JSON format | ❌ TODO | `src/readers/bsejson.rs` |

**Completed readers handle**: Electron shells, ECP potentials, general contractions, SP combined shells.

---

## Priority 5: Missing Writers

| Format | Description | Status | File |
|--------|-------------|--------|------|
| `molcas_library` | Molcas basis library format | ❌ TODO | `src/writers/molcas.rs` |
| `libmol` | Molpro system library | ❌ TODO | `src/writers/libmol.rs` |
| `bsedebug` | BSE debug format | ❌ TODO | `src/writers/bsedebug.rs` |
| `bsejson` | Native JSON output | ❌ TODO | `src/writers/bsejson.rs` |
| `ricdwrap` | Wrapper for acCD auxiliary basis | ❌ TODO | `src/writers/ricdwrap.rs` |

---

## Priority 6: Missing Manipulation Functions

| Function | Description | Status | File |
|----------|-------------|--------|------|
| `remove_primitive` | Remove a primitive by index | ❌ TODO | `src/manip.rs` |
| `truhlar_calendarize` | Create "month" basis sets (jul, jun, etc.) | ❌ TODO | `src/manip.rs` |
| `merge_element_data` | Merge basis data from multiple sources | ❌ TODO | `src/manip.rs` |
| `create_element_data` | Helper for creating element data | ❌ TODO | `src/manip.rs` |

---

## Priority 7: Utility Modules

| Module | Description | Status | Notes |
|--------|-------------|--------|-------|
| `convert.py` equivalent | Format conversion convenience | ❌ TODO | `convert_formatted_basis_str`, `convert_formatted_basis_file` |
| `validator.py` equivalent | Validate basis set data | ❌ TODO | Schema validation |
| `skel.rs` | Create skeleton basis structures | ❌ TODO | Helper for creating empty basis templates |
| `fileio.rs` | File I/O utilities | ❌ TODO | Reading notes, metadata, references files |
| `memo.rs` | Memoization utilities | ⚠️ Partial | Rust uses `#[cached]` macro instead |

---

## Priority 8: Advanced Features (Lower Priority)

| Feature | Description | Status | Notes |
|---------|-------------|--------|-------|
| Bundle creation | Create archives of all basis sets | ❌ TODO | ZIP/tar.bz2 bundle generation |
| CLI interface | Command-line tool | ❌ TODO | `bse` command equivalents |
| Curation utilities | Diff, compare, graph | ❌ TODO | Mostly for development/maintenance |

---

## Implementation Notes

1. **Follow Python structure**: Keep close to Python implementation for maintainability
2. **Add tests**: Each new function should have corresponding tests
3. **Update prelude**: Export new public functions in `src/prelude.rs`
4. **Documentation**: Add docstrings with examples for public API
5. **Case insensitive**: Maintain case-insensitive handling for names and formats

---

## Progress Tracking

- Last updated: 2026-04-01
- Completed items: 18 (Priority 1, 2, and 4 mostly complete)
- Total items tracked: ~40