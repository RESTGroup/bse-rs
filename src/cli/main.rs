//! Command-line interface for the Basis Set Exchange in Rust.
//!
//! This CLI is functionally equivalent to the Python BSE CLI.

use bse::cli::handlers::*;
use bse::is_dir_format;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// Basis Set Exchange CLI - retrieve, manipulate, and convert Gaussian-type
/// orbital basis sets for computational chemistry.
#[derive(Parser)]
#[command(name = "bse-rs")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Override which data directory to use
    #[arg(short = 'd', long = "data-dir", global = true, value_name = "PATH")]
    data_dir: Option<PathBuf>,

    /// Output to given file rather than stdout
    #[arg(short = 'o', long = "output", global = true, value_name = "PATH")]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Output a list of basis set formats that can be written
    ListWriterFormats(ListFormatsArgs),

    /// Output a list of basis set formats that can be read
    ListReaderFormats(ListFormatsArgs),

    /// Output a list of all available reference formats and descriptions
    ListRefFormats(ListFormatsArgs),

    /// Output a list of all available roles and descriptions
    ListRoles(ListFormatsArgs),

    /// Output the default data directory of this package
    GetDataDir,

    /// Output a list of all available basis sets and descriptions
    ListBasisSets(ListBasisSetsArgs),

    /// Output a list of all available basis set families
    ListFamilies,

    /// Lookup a companion/auxiliary basis by primary basis and role
    LookupByRole(LookupByRoleArgs),

    /// Output a formatted basis set
    GetBasis(GetBasisArgs),

    /// Output references for a basis set
    GetRefs(GetRefsArgs),

    /// Output general info and metadata for a basis set
    GetInfo(GetInfoArgs),

    /// Output the notes for a basis set
    GetNotes(GetNotesArgs),

    /// Output the family of a basis set
    GetFamily(GetFamilyArgs),

    /// Output a list of all available versions of a basis set
    GetVersions(GetVersionsArgs),

    /// Get the notes of a family of basis sets
    GetFamilyNotes(GetFamilyNotesArgs),

    /// Convert basis set files from one format to another
    ConvertBasis(ConvertBasisArgs),

    /// Form AutoAux auxiliary basis
    AutoauxBasis(AutoauxBasisArgs),

    /// Form AutoABS auxiliary basis
    AutoabsBasis(AutoauxBasisArgs),

    /// Generate or install shell completion scripts
    Completion(CompletionArgs),
}

// ============================================================================
// Argument structs for each subcommand
// ============================================================================

/// Simple listing arguments (just -n for no description)
#[derive(clap::Args)]
struct ListFormatsArgs {
    /// Print only the names without descriptions
    #[arg(short = 'n', long = "no-description")]
    no_description: bool,
}

/// List basis sets with optional filters
#[derive(clap::Args)]
struct ListBasisSetsArgs {
    /// Print only the basis set names without descriptions
    #[arg(short = 'n', long = "no-description")]
    no_description: bool,

    /// Limit the basis set list to only the specified family
    #[arg(short = 'f', long = "family")]
    family: Option<String>,

    /// Limit the basis set list to only the specified role
    #[arg(short = 'r', long = "role")]
    role: Option<String>,

    /// Limit the basis set list to only basis sets whose name contains the
    /// specified substring
    #[arg(short = 's', long = "substr")]
    substr: Option<String>,

    /// Limit the basis set list to only basis sets that contain all the given
    /// elements
    #[arg(short = 'e', long = "elements")]
    elements: Option<String>,
}

/// Lookup by role arguments
#[derive(clap::Args)]
struct LookupByRoleArgs {
    /// Name of the primary basis we want the auxiliary basis for
    basis: String,

    /// Role of the auxiliary basis to look for
    role: String,
}

/// Get basis set with all manipulation options
#[derive(clap::Args)]
struct GetBasisArgs {
    /// Name of the basis set to output
    basis: String,

    /// Which format to output the basis set as
    fmt: String,

    /// Which elements of the basis set to output. Default is all defined in the
    /// given basis
    #[arg(long = "elements")]
    elements: Option<String>,

    /// Which version of the basis set to output. Default is the latest version
    #[arg(long = "basis-version")]
    version: Option<String>,

    /// Do not output the header at the top
    #[arg(long = "noheader")]
    noheader: bool,

    /// Remove general contractions
    #[arg(long = "unc-gen")]
    unc_gen: bool,

    /// Remove combined sp, spd, ... contractions
    #[arg(long = "unc-spdf")]
    unc_spdf: bool,

    /// Remove segmented contractions
    #[arg(long = "unc-seg")]
    unc_seg: bool,

    /// Remove free primitives
    #[arg(long = "rm-free")]
    rm_free: bool,

    /// Optimize general contractions
    #[arg(long = "opt-gen")]
    opt_gen: bool,

    /// Make the basis set as generally-contracted as possible
    #[arg(long = "make-gen")]
    make_gen: bool,

    /// Augment with n steep functions
    #[arg(long = "aug-steep", default_value = "0")]
    aug_steep: i32,

    /// Augment with n diffuse functions
    #[arg(long = "aug-diffuse", default_value = "0")]
    aug_diffuse: i32,

    /// Instead of the orbital basis, get an automatically formed auxiliary
    /// basis (0=orbital, 1=autoaux, 2=autoabs)
    #[arg(long = "get-aux", default_value = "0")]
    get_aux: i32,
}

/// Get references arguments
#[derive(clap::Args)]
struct GetRefsArgs {
    /// Name of the basis set to output the references for
    basis: String,

    /// Which format to output the references as
    reffmt: String,

    /// Which elements to output the references for. Default is all defined in
    /// the given basis
    #[arg(long = "elements")]
    elements: Option<String>,

    /// Which version of the basis set to get the references for
    #[arg(long = "basis-version")]
    version: Option<String>,
}

/// Get info arguments
#[derive(clap::Args)]
struct GetInfoArgs {
    /// Name of the basis set to output the info for
    basis: String,
}

/// Get notes arguments
#[derive(clap::Args)]
struct GetNotesArgs {
    /// Name of the basis set to output the notes for
    basis: String,
}

/// Get family arguments
#[derive(clap::Args)]
struct GetFamilyArgs {
    /// Name of the basis set to output the family for
    basis: String,
}

/// Get versions arguments
#[derive(clap::Args)]
struct GetVersionsArgs {
    /// Name of the basis set to list the versions of
    basis: String,

    /// Print only the version numbers without descriptions
    #[arg(short = 'n', long = "no-description")]
    no_description: bool,
}

/// Get family notes arguments
#[derive(clap::Args)]
struct GetFamilyNotesArgs {
    /// The basis set family to get the notes of
    family: String,
}

/// Convert basis arguments
#[derive(clap::Args)]
struct ConvertBasisArgs {
    /// Basis set file to convert
    input_file: PathBuf,

    /// Converted basis set file
    output_file: PathBuf,

    /// Input format (default: autodetected from input filename)
    #[arg(long = "in-fmt")]
    in_fmt: Option<String>,

    /// Output format (default: autodetected from output filename)
    #[arg(long = "out-fmt")]
    out_fmt: Option<String>,

    /// Make the basis set as generally-contracted as possible
    #[arg(long = "make-gen")]
    make_gen: bool,
}

/// AutoAux/AutoABS basis arguments
#[derive(clap::Args)]
struct AutoauxBasisArgs {
    /// Orbital basis to load
    input_file: PathBuf,

    /// Output basis file
    output_file: PathBuf,

    /// Input format (default: autodetected from input filename)
    #[arg(long = "in-fmt")]
    in_fmt: Option<String>,

    /// Output format (default: autodetected from output filename)
    #[arg(long = "out-fmt")]
    out_fmt: Option<String>,
}

/// Shell completion arguments
#[derive(clap::Args)]
struct CompletionArgs {
    /// Shell to generate completion for (default: auto-detect from environment)
    #[arg(value_enum)]
    shell: Option<Shell>,

    /// Install completion script to the appropriate location for your shell
    #[arg(short = 'i', long = "install")]
    install: bool,
}

// ============================================================================
// Completion helpers
// ============================================================================

/// Detect the current shell from environment variables.
fn detect_shell() -> Option<Shell> {
    // Check $SHELL environment variable
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return Some(Shell::Zsh);
        }
        if shell.contains("bash") {
            return Some(Shell::Bash);
        }
        if shell.contains("fish") {
            return Some(Shell::Fish);
        }
        if shell.contains("elvish") {
            return Some(Shell::Elvish);
        }
    }

    // Check PowerShell on Windows
    if std::env::var("PSModulePath").is_ok() {
        return Some(Shell::PowerShell);
    }

    None
}

/// Get the default completion file path for a shell.
fn get_completion_path(shell: Shell) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let home = PathBuf::from(home);

    match shell {
        Shell::Bash => {
            // Bash uses XDG_DATA_HOME or ~/.local/share/bash-completion/completions
            let xdg = std::env::var("XDG_DATA_HOME").ok();
            let base = xdg.map(PathBuf::from).unwrap_or_else(|| home.join(".local/share"));
            Some(base.join("bash-completion/completions/bse-rs"))
        },
        Shell::Zsh => {
            // Zsh uses ~/.zfunc/ or fpath
            Some(home.join(".zfunc/_bse-rs"))
        },
        Shell::Fish => {
            // Fish uses ~/.config/fish/completions/
            Some(home.join(".config/fish/completions/bse-rs.fish"))
        },
        Shell::Elvish => Some(home.join(".local/share/elvish/lib/bse-rs.elv")),
        Shell::PowerShell => {
            // PowerShell uses a scripts directory
            let profile = std::env::var("USERPROFILE").ok()?;
            Some(PathBuf::from(profile).join("Documents/PowerShell/bse-rs.ps1"))
        },
        _ => None,
    }
}

/// Get instructions for enabling completion after installation.
fn get_completion_instructions(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => {
            "To enable completion, restart your shell or run:\n  source ~/.local/share/bash-completion/completions/bse-rs"
        }
        Shell::Zsh => {
            "To enable completion, add this to your ~/.zshrc:\n  fpath+=~/.zfunc\n  autoload -U compinit && compinit\nThen restart your shell or run: source ~/.zshrc"
        }
        Shell::Fish => {
            "Completion will be automatically enabled on next shell start."
        }
        Shell::Elvish => {
            "To enable completion, add this to your ~/.elvish/rc.elv:\n  eval (bse-rs completion elvish)"
        }
        Shell::PowerShell => {
            "To enable completion, add this to your PowerShell profile:\n  . $HOME\\Documents\\PowerShell\\bse-rs.ps1"
        }
        _ => "Unknown shell. Please refer to your shell's documentation.",
    }
}

/// Handle the completion subcommand.
fn handle_completion(shell: Option<Shell>, install: bool) -> Result<String, bse::BseError> {
    use bse::{bse_raise, BseError};

    let shell = match shell.or_else(detect_shell) {
        Some(s) => s,
        None => {
            return bse_raise!(
                ValueError,
                "Could not auto-detect shell. Please specify a shell: bash, zsh, fish, powershell, elvish"
            );
        },
    };

    if install {
        // Install completion to the appropriate location
        let path = match get_completion_path(shell) {
            Some(p) => p,
            None => {
                return bse_raise!(ValueError, "Could not determine completion installation path for {:?}", shell);
            },
        };

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| BseError::IOError(format!("Failed to create directory {}: {}", parent.display(), e)))?;
        }

        // Generate and write completion
        let mut file = std::fs::File::create(&path)
            .map_err(|e| BseError::IOError(format!("Failed to create file {}: {}", path.display(), e)))?;

        clap_complete::generate(shell, &mut Cli::command(), "bse-rs", &mut file);

        Ok(format!("Completion installed to: {}\n\n{}", path.display(), get_completion_instructions(shell)))
    } else {
        // Just print completion to stdout
        clap_complete::generate(shell, &mut Cli::command(), "bse-rs", &mut std::io::stdout());
        Ok(String::new())
    }
}

// ============================================================================
// Main entry point
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let data_dir_str = cli.data_dir.as_ref().map(|p| p.to_string_lossy().to_string());

    // Check if this is a directory format (needed for output handling)
    let is_dir_output = match &cli.command {
        Commands::GetBasis(args) => is_dir_format(&args.fmt),
        Commands::ConvertBasis(args) => args.out_fmt.as_ref().map(|f| is_dir_format(f)).unwrap_or(false),
        _ => false,
    };

    // Handle the command and get output
    let result = match cli.command {
        // Simple listings
        Commands::ListWriterFormats(args) => handle_list_writer_formats(args.no_description),
        Commands::ListReaderFormats(args) => handle_list_reader_formats(args.no_description),
        Commands::ListRefFormats(args) => handle_list_ref_formats(args.no_description),
        Commands::ListRoles(args) => handle_list_roles(args.no_description),
        Commands::GetDataDir => handle_get_data_dir(),
        Commands::ListFamilies => handle_list_families(data_dir_str),

        // Basis set listings with filters
        Commands::ListBasisSets(args) => handle_list_basis_sets(
            args.substr,
            args.family,
            args.role,
            args.elements,
            data_dir_str,
            args.no_description,
        ),

        // Lookup by role
        Commands::LookupByRole(args) => handle_lookup_by_role(args.basis, args.role, data_dir_str),

        // Get basis set
        Commands::GetBasis(args) => handle_get_basis(
            args.basis,
            args.fmt,
            args.elements,
            args.version,
            args.noheader,
            args.unc_gen,
            args.unc_spdf,
            args.unc_seg,
            args.rm_free,
            args.opt_gen,
            args.make_gen,
            args.aug_diffuse,
            args.aug_steep,
            args.get_aux,
            data_dir_str,
            cli.output.clone(),
        ),

        // Get references
        Commands::GetRefs(args) => handle_get_refs(args.basis, args.reffmt, args.elements, args.version, data_dir_str),

        // Get info
        Commands::GetInfo(args) => handle_get_info(args.basis, data_dir_str),

        // Get notes
        Commands::GetNotes(args) => handle_get_notes(args.basis, data_dir_str),

        // Get family
        Commands::GetFamily(args) => handle_get_family(args.basis, data_dir_str),

        // Get versions
        Commands::GetVersions(args) => handle_get_versions(args.basis, data_dir_str, args.no_description),

        // Get family notes
        Commands::GetFamilyNotes(args) => handle_get_family_notes(args.family, data_dir_str),

        // Convert basis
        Commands::ConvertBasis(args) => {
            handle_convert_basis(args.input_file, args.output_file, args.in_fmt, args.out_fmt, args.make_gen)
        },

        // AutoAux basis
        Commands::AutoauxBasis(args) => {
            handle_autoaux_basis(args.input_file, args.output_file, args.in_fmt, args.out_fmt)
        },

        // AutoABS basis
        Commands::AutoabsBasis(args) => {
            handle_autoabs_basis(args.input_file, args.output_file, args.in_fmt, args.out_fmt)
        },

        // Shell completion
        Commands::Completion(args) => handle_completion(args.shell, args.install),
    };

    // Handle result
    match result {
        Ok(output) => {
            // For directory formats, output is a success message (already written)
            // For regular formats, write to file or stdout
            if is_dir_output {
                // Directory format - output is already written, just print message
                if !output.is_empty() {
                    println!("{}", output);
                }
            } else if let Some(output_path) = cli.output {
                if let Err(e) = std::fs::write(&output_path, output + "\n") {
                    eprintln!("Error writing to {}: {}", output_path.display(), e);
                    std::process::exit(1);
                }
            } else if !output.is_empty() {
                println!("{}", output);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    }
}
