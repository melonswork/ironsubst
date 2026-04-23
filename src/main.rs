use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use ironsubst::{envfile, eval::Restrictions, process};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about = "Environment variables substitution", long_about = None)]
pub struct Args {
    /// Input template file (defaults to stdin)
    #[arg(short, long)]
    input: Option<String>,

    /// Output file (defaults to stdout). Written atomically via a temp file + rename.
    #[arg(short, long)]
    output: Option<String>,

    /// Do not substitute variables whose name starts with a digit (e.g. $1, ${12})
    #[arg(long)]
    no_digit: bool,

    /// Fail if a variable is not explicitly set in the environment.
    /// Fallback operators (e.g. ${X:-default}) are respected — only bare unset variables error.
    #[arg(long)]
    require_values: bool,

    /// Fail if a variable is set to an empty string (or unset with no fallback).
    #[arg(long)]
    require_nonempty_values: bool,

    /// Stop on the first error instead of collecting all errors
    #[arg(short, long)]
    fail_fast: bool,

    /// Only substitute variables whose names start with this prefix.
    /// Variables that do not match are left verbatim in the output.
    /// Example: --prefix MYAPP_ substitutes $MYAPP_HOST but leaves $OTHER unchanged.
    #[arg(long, value_name = "PREFIX")]
    prefix: Option<String>,

    /// Load environment variables from a .env file.
    /// May be specified multiple times; later files override earlier ones.
    /// File variables are merged on top of the shell environment.
    /// Supports: KEY=VALUE, export KEY=VALUE, quoted values, # comments.
    #[arg(long, value_name = "FILE", action = clap::ArgAction::Append)]
    env_file: Vec<String>,

    /// Ignore the current shell environment entirely.
    /// When set, the substitution environment is built solely from --env-file sources.
    /// Useful for reproducible renders that must not depend on the caller's shell state.
    #[arg(long, requires = "env_file")]
    ignore_env: bool,

    /// Print a man page in roff format to stdout.
    /// Pipe to `man -l -` or install to $HOME/.local/share/man/man1/.
    #[arg(long, hide = true)]
    generate_man_page: bool,

    /// Inline template string (use `--` to separate from flags)
    #[arg(last = true)]
    explicit_input: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate shell completions and print to stdout.
    ///
    /// Supported shells: bash, zsh, fish, powershell
    ///
    /// Examples:
    ///   ironsubst completions bash >> ~/.bash_completion
    ///   ironsubst completions zsh > ~/.zfunc/_ironsubst
    ///   ironsubst completions fish > ~/.config/fish/completions/ironsubst.fish
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

fn main() {
    let args = Args::parse();

    // --- Subcommands ---
    if let Some(Command::Completions { shell }) = args.command {
        let mut cmd = Args::command();
        clap_complete::generate(shell, &mut cmd, "ironsubst", &mut io::stdout());
        return;
    }

    // --- Hidden plumbing flags ---
    if args.generate_man_page {
        let cmd = Args::command();
        let man = clap_mangen::Man::new(cmd);
        let mut buf = Vec::new();
        man.render(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error generating man page: {}", e);
            std::process::exit(1);
        });
        io::stdout().write_all(&buf).unwrap_or_else(|e| {
            eprintln!("Error writing man page: {}", e);
            std::process::exit(1);
        });
        return;
    }

    // --- Normal substitution mode ---
    let mut input_data = String::new();
    if let Some(explicit) = args.explicit_input {
        input_data = explicit;
    } else if let Some(input_file) = args.input {
        let mut file = File::open(&input_file).unwrap_or_else(|e| {
            eprintln!("Error opening input file {}: {}", input_file, e);
            std::process::exit(1);
        });
        file.read_to_string(&mut input_data).unwrap_or_else(|e| {
            eprintln!("Error reading input file {}: {}", input_file, e);
            std::process::exit(1);
        });
    } else {
        io::stdin()
            .read_to_string(&mut input_data)
            .unwrap_or_else(|e| {
                eprintln!("Error reading from stdin: {}", e);
                std::process::exit(1);
            });
    }

    // Build the environment.
    // With --ignore-env: start from an empty map (shell env is excluded).
    // Otherwise: start from the current shell environment.
    // Either way, --env-file entries are layered on top in order.
    let mut env: HashMap<String, String> = if args.ignore_env {
        HashMap::new()
    } else {
        std::env::vars().collect()
    };
    if !args.env_file.is_empty() {
        envfile::load_env_files(&args.env_file, &mut env).unwrap_or_else(|e| {
            eprintln!("Error loading env file: {e}");
            std::process::exit(1);
        });
    }

    let restrictions = Restrictions {
        require_values: args.require_values,
        require_nonempty_values: args.require_nonempty_values,
    };

    match process(
        &input_data,
        &env,
        restrictions,
        args.no_digit,
        args.fail_fast,
        args.prefix.as_deref(),
    ) {
        Ok(result) => {
            if let Some(output_file) = args.output {
                // Write atomically: write to a temp file in the same directory,
                // then rename() it over the destination. On POSIX this is atomic,
                // so a kill/OOM/disk-full mid-write cannot leave a partial file.
                let dest = Path::new(&output_file);
                let dir = dest.parent().unwrap_or_else(|| Path::new("."));
                let mut tmp = tempfile::NamedTempFile::new_in(dir).unwrap_or_else(|e| {
                    eprintln!("Error creating temp file near {}: {}", output_file, e);
                    std::process::exit(1);
                });
                tmp.write_all(result.as_bytes()).unwrap_or_else(|e| {
                    eprintln!("Error writing to temp file: {}", e);
                    std::process::exit(1);
                });
                tmp.persist(dest).unwrap_or_else(|e| {
                    eprintln!("Error writing to output file {}: {}", output_file, e);
                    std::process::exit(1);
                });
            } else {
                print!("{}", result);
            }
        }
        Err(err) => {
            eprintln!("{}\n", err);
            std::process::exit(1);
        }
    }
}
