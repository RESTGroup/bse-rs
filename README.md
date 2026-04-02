# Basis Set Exchange in Rust (bse-rs)

[![Crates.io](https://img.shields.io/crates/v/bse.svg)](https://crates.io/crates/bse)
[![Documentation](https://docs.rs/bse/badge.svg)](https://docs.rs/bse)
[![License](https://img.shields.io/crates/l/bse.svg)](https://crates.io/crates/bse)
[![Rust Version](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)

A Rust library and CLI tool for retrieving, manipulating, and converting Gaussian-type orbital (GTO) basis sets for computational chemistry. This is a complete reimplementation of the Python [Basis Set Exchange](https://github.com/MolSSI-BSE/basis_set_exchange/) library, providing full API compatibility, with some additional features.

## Overview

**bse-rs** provides:

- **Library API**: Full Rust API for basis set retrieval, manipulation, and format conversion
- **CLI Tool**: Feature-complete command-line interface with shell completion support
- **Remote Access**: Optional REST API client for fetching basis sets from basissetexchange.org
- **Format Conversion**: 27+ output formats and 18+ input formats for quantum chemistry software
- **Basis Set Manipulation**: Uncontracting, optimization, augmentation, and auxiliary basis generation

This implementation adds several distinct features beyond the Python reference:

- **Support to REST**: [REST](https://gitee.com/restgroup/rest) (Rust-based Electronic Structure Toolkit) format supported for both reading and writing
- **Seamless Truhlar Calendar Support**: Request calendar basis sets (e.g., `jul-cc-pVTZ`, `maug-cc-pVDZ`) directly through the standard API and CLI without calling separate functions
- **Format Auto-detection in Conversion**: CLI automatically detects format from file extensions
- **TOML Configuration**: For API usage, basis set arguments can be parsed from TOML configuration files
- **Support of Directory Read/Write**: Read/write basis sets as directories with one file per element, in addition to single-file formats

## Installation

### From Crates.io

```bash
cargo install bse
```

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bse = "0.1"
```

For remote API access:

```toml
[dependencies]
bse = { version = "0.1", features = ["remote"] }
```

### Build from Source

```bash
git clone https://github.com/RESTGroup/bse-rs.git
cd bse-rs
cargo build --release
```

## Quick Start

### Basic CLI Usage

For more details on CLI options, don't bother `bse-rs --help`.

```bash
# List available basis sets
bse-rs list-basis-sets

# Get basis set in NWChem format
bse-rs get-basis def2-TZVP nwchem --elements "H,C,N,F-Mg,47-55"

# Get basis set with manipulations in Gaussian format
# This specific case is uncontract all segmented contractions and add 1 diffuse function
bse-rs get-basis cc-pVTZ gaussian --unc-seg --aug-diffuse 1

# Get full basis set to directory in REST format (one file per element)
bse-rs get-basis def2-TZVP rest

# Output to directory in Gaussian format (one file per element)
bse-rs get-basis def2-TZVP dir-gaussian94 -o ./basis_dir

# Convert between formats (auto-detects from extension)
bse-rs convert-basis input.nw output.gbs

# Convert between formats (use specific format)
bse-rs convert-basis input.nw basis_dir --out-fmt rest

# Get Truhlar calendar basis set directly
bse-rs get-basis jul-cc-pVTZ nwchem --elements "Zn"

# Generate shell completion
bse-rs completion bash --install
```

### Download Basis Data

*Note that if remote access is allowed, the library will fetch data directly from <https://basissetexchange.org>. This is the default behavior if the `remote` feature is enabled. In this case, no local data is required.*

If you have the Python BSE package installed, it will use that data. Be sure the `basis_set_exchange` Python package is installed and accessible in your environment.

```bash
pip install basis_set_exchange
```

Otherwise, you can specify the data directory using the `BSE_DATA_DIR` environment variable or the `--data-dir` CLI option. The original data can be accessable from the [Python BSE repository](https://github.com/MolSSI-BSE/basis_set_exchange/tree/master/basis_set_exchange/data):

```bash
export CURRENT_DIR=$(pwd)
git clone https://github.com/MolSSI-BSE/basis_set_exchange.git
export BSE_DATA_DIR="$CURRENT_DIR/basis_set_exchange/basis_set_exchange/data"
```

### Library API

```rust
use bse::prelude::*;

// Get basis set as structured object
let args = BseGetBasisArgsBuilder::default().elements("H, C-O".to_string()).build().unwrap();
let basis = get_basis("cc-pVTZ", args);
println!("Basis: {} ({})", basis.name, basis.family);

// use TOML configuration for arguments
let args_string = r#"
    elements = "H, C-O"
    augment_diffuse = 1
"#;
let basis = get_basis("cc-pVTZ", args_string);
println!("Basis: {} ({})", basis.name, basis.family);

// Get formatted output for quantum chemistry software
let args = BseGetBasisArgsBuilder::default().elements("H, O".to_string()).header(true).build().unwrap();
let output = get_formatted_basis("sto-3g", "nwchem", args);
println!("{}", output);

// Read basis from file
// assumes basis.nw exists and is in NWChem format
let content = std::fs::read_to_string("basis.nw").unwrap();
let basis_minimal = read_formatted_basis_str(&content, "nwchem");
println!("Basis: {}", basis_minimal.name);

// Apply manipulations
let args = BseGetBasisArgsBuilder::default().uncontract_general(true).augment_diffuse(2).build().unwrap();
let basis = get_basis("def2-SVP", args);
println!("Basis: {} ({})", basis.name, basis.family);

// Get Truhlar calendar basis directly (seamless integration)
let basis = get_basis("jul-cc-pVTZ", BseGetBasisArgs::default());
println!("Basis: {} ({})", basis.name, basis.family);
let basis = get_basis("maug-cc-pVDZ", BseGetBasisArgs::default()); // Auto-selects jun for DZ
println!("Basis: {} ({})", basis.name, basis.family);
```

## Features and Configuration

### Data Source Configuration

The library supports multiple data sources with automatic detection:

1. **Python BSE Package**: Auto-detects if `basis_set_exchange` is installed via pip
2. **Environment Variable**: Set `BSE_DATA_DIR` to point to BSE data directory
3. **Runtime Specification**: Use `specify_bse_data_dir()` function
4. **Remote API**: Fetch directly from basissetexchange.org (requires `remote` feature)

#### BSE_DATA_DIR - Local Data Directory

```bash
# Environment variable method
export BSE_DATA_DIR=/path/to/basis_set_exchange/basis_set_exchange/data
```

#### BSE_REMOTE - Default Data Source

The `BSE_REMOTE` environment variable controls the default data source behavior:

```bash
# Use local data directory only
export BSE_REMOTE=local

# Use remote REST API only (requires remote feature)
export BSE_REMOTE=remote

# Try local first, fallback to remote if local fails (default)
export BSE_REMOTE=auto
```

Supported values:
- `local`, `0`, `false`, `no`: Use local data directory only
- `remote`, `1`, `true`, `yes`: Use remote REST API only
- `auto`: Try local first, fallback to remote (default)

#### BSE_TIMEOUT - Request Timeout

Timeout in seconds for remote API requests (default: 10). Only applies when
using `remote` or `auto` source with the `remote` feature enabled.

```bash
export BSE_TIMEOUT=30  # 30 second timeout
```

#### BSE_WARN_LOCAL_NOTFOUND - Fallback Warning

Control warning message when falling back from local to remote in `auto` mode
(default: true/warning enabled).

```bash
export BSE_WARN_LOCAL_NOTFOUND=0  # Suppress fallback warning
```

#### CLI Source Override

The `--source` option overrides the `BSE_REMOTE` environment variable:

```bash
# Override to use remote
bse-rs --source remote get-basis cc-pVTZ nwchem

# Override to use auto (tries local, then remote)
bse-rs --source auto get-basis cc-pVTZ nwchem
```

### Supported Output Formats (Writers)

| Name | Extension | Aliases | Display |
|------|-----------|---------|---------|
| acesii | .acesii | | ACES II |
| bdf | .bdf | | BDF |
| bsedebug | .bse | | BSE Debug |
| cfour | .c4bas | c4bas | CFOUR |
| cp2k | .cp2k | | CP2K |
| crystal | .crystal | | Crystal |
| dalton | .dalton | | Dalton |
| demon2k | .d2k | d2k | deMon2K |
| fhiaims | .fhiaims | | FHI-aims |
| gamess_uk | .bas | | GAMESS UK |
| gamess_us | .bas | | GAMESS US |
| gaussian94 | .gbs | g94, gaussian, gau | Gaussian |
| gaussian94lib | .gbs | g94lib | Gaussian, system library |
| jaguar | .jaguar | | Jaguar |
| json | .json | bsejson | JSON |
| libmol | .libmol | | Molpro system library |
| molcas | .molcas | | Molcas |
| molcas_library | .molcas | | Molcas basis library |
| molpro | .mpro | mpro | Molpro |
| nwchem | .nw | nw | NWChem |
| orca | .orca | | ORCA |
| pqs | .pqs | | PQS |
| psi4 | .gbs | | Psi4 |
| qchem | .qchem | | Q-Chem |
| qcschema | .json | | QCSchema |
| rest | .json | | REST (directory only format) |
| ricdwrap | .ricdwrap | | Wrapper for generating acCD auxiliary basis sets with OpenMolcas |
| turbomole | .tm | tm | Turbomole |
| veloxchem | .vlx | vlx | VeloxChem |
| xtron | .gbs | | xTron |

### Supported Input Formats (Readers)

| Name | Extension | Aliases | Display |
|------|-----------|---------|---------|
| cfour | .c4bas | | CFOUR |
| cp2k | .cp2k | | CP2K |
| crystal | .crystal | | Crystal |
| dalton | .mol | mol | Dalton |
| demon2k | .d2k | d2k | deMon2k |
| gamess_us | .bas | | GAMESS US |
| gaussian94 | .gbs | gaussian, g94, gbs, gau | Gaussian94 |
| gbasis | .gbasis | | GBasis |
| genbas | .genbas | | Genbas |
| json | .json | bsejson | JSON |
| libmol | .libmol | | Molpro system library |
| molcas | .molcas | | Molcas |
| molcas_library | .molcas | | Molcas basis library |
| molpro | .mpro | mpro | Molpro |
| nwchem | .nw | nw | NWChem |
| rest | .json | | REST (directory only format) |
| ricdlib | .ricdlib | ricd | MolCAS RICDlib |
| turbomole | .tm | tm | Turbomole |
| veloxchem | .vlx | vlx | VeloxChem |

### Basis Set Manipulations

| Option | Description |
|--------|-------------|
| `uncontract_general` | Remove general contractions by duplicating primitives |
| `uncontract_spdf` | Split combined shells (sp, spd, spdf) into separate shells |
| `uncontract_segmented` | Fully uncontract (each primitive becomes separate shell) |
| `make_general` | Make basis set as generally-contracted as possible |
| `optimize_general` | Optimize general contractions by removing redundant functions |
| `remove_free_primitives` | Remove uncontracted (free) primitives |
| `augment_diffuse` | Add n diffuse functions via even-tempered extrapolation |
| `augment_steep` | Add n steep functions via even-tempered extrapolation |
| `get_aux` | Generate auxiliary basis: 0=orbital, 1=AutoAux, 2=AutoABS |

### Truhlar Calendar Basis Sets

Request calendar basis sets directly through the standard API:

```rust
// All month prefixes supported
get_basis("jul-cc-pVTZ", args);  // July - all diffuse
get_basis("jun-cc-pVTZ", args);  // June - s,p diffuse
get_basis("may-cc-pVTZ", args);  // May - s diffuse
get_basis("apr-cc-pVTZ", args);  // April - no diffuse

// maug automatically selects based on zeta level
get_basis("maug-cc-pVDZ", args); // -> jun-cc-pVDZ
get_basis("maug-cc-pVTZ", args); // -> may-cc-pVTZ
get_basis("maug-cc-pVQZ", args); // -> apr-cc-pVQZ
```

### Reference/Citation Formats

```bash
# Get BibTeX references
bse-rs get-refs cc-pVTZ bib --elements "H,C"

# Available formats: bib, txt, ris, endnote, json
```

### Metadata Queries

```bash
# List basis sets with filters
bse-rs list-basis-sets --family dunning --substr aug
bse-rs list-basis-sets --role jkfit --elements "H-C"

# Get basis set info
bse-rs get-info cc-pVTZ

# Lookup auxiliary basis by role
bse-rs lookup-by-role cc-pVTZ jkfit
```

## API Documentation

Full API documentation is available at [docs.rs/bse](https://docs.rs/bse).

### Key Functions

| Function | Description |
|----------|-------------|
| `get_basis` | Retrieve basis set by name |
| `get_formatted_basis` | Get formatted output for specific software |
| `read_formatted_basis_str` | Parse basis set from formatted string |
| `write_formatted_basis_str` | Convert basis set to specific format |
| `get_metadata` | Get all basis set metadata |
| `filter_basis_sets` | Filter basis sets by family, role, elements |
| `get_references` | Get citations for a basis set |
| `get_family_notes` | Get notes for a basis set family |
| `lookup_basis_by_role` | Find auxiliary basis by role |

### Data Structures

| Struct | Description |
|--------|-------------|
| `BseBasis` | Complete basis set with all metadata |
| `BseBasisElement` | Per-element basis data (shells, ECPs) |
| `BseElectronShell` | Individual shell (angular momentum, exponents, coefficients) |
| `BseGetBasisArgs` | Arguments for basis set retrieval and manipulation |
| `BseFilterArgs` | Arguments for filtering basis sets |

## CLI Reference

```bash
bse-rs [OPTIONS] <COMMAND>

Commands:
  list-basis-sets      List available basis sets
  list-writer-formats  List output formats
  list-reader-formats  List input formats
  list-families        List basis set families
  list-roles           List basis set roles
  get-basis            Get formatted basis set
  get-info             Get basis set metadata
  get-refs             Get citations
  get-versions         List versions of a basis set
  get-notes            Get basis set notes
  get-family           Get basis set family
  get-family-notes     Get family notes
  lookup-by-role       Find auxiliary basis
  convert-basis        Convert between formats
  autoaux-basis        Generate AutoAux basis
  autoabs-basis        Generate AutoABS basis
  get-data-dir         Show data directory path
  completion           Generate shell completion

Options:
  -d, --data-dir <PATH>  Override data directory
  --source <SOURCE>      Data source: local, remote, auto
  -o, --output <PATH>    Output file path
```

## Comparison with Python BSE

| Feature | Python BSE | bse-rs |
|---------|------------|--------|
| Basis set retrieval | ✓ | ✓ |
| Format conversion | ✓ | ✓ |
| CLI tool | ✓ | ✓ (with completion) |
| REST API access | ✓ | ✓ (optional) |
| Truhlar calendar | Separate function | Integrated in get_basis |
| Format auto-detection | Partial | Yes |
| TOML configuration | No | Yes |

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test test_get_basis
cargo test --test test_read_basis

# Run with verbose output
cargo test -- --nocapture
```

Tests compare output against Python BSE v0.11 reference files for compatibility verification.

## References

- [MolSSI Basis Set Exchange](https://www.basissetexchange.org)
- [Python BSE Repository](https://github.com/MolSSI-BSE/basis_set_exchange)
- [BSE API Documentation](https://www.basissetexchange.org/api)

## License

MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

This project is a Rust reimplementation of the Python [Basis Set Exchange](https://github.com/MolSSI-BSE/basis_set_exchange/) library by MolSSI. The basis set data is provided by the Python BSE project.

This project is supported by [REST](https://gitee.com/restgroup/rest) (Rust-based Electronic Structure Toolkit).

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues on the [GitHub repository](https://github.com/RESTGroup/bse-rs).
