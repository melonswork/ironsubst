use clap::Parser;
use ironsubst::{eval::Restrictions, process};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(author, version, about = "Environment variables substitution", long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    no_digit: bool,

    #[arg(long)]
    require_explicit_values: bool,

    #[arg(long)]
    require_any_values: bool,

    #[arg(long)]
    require_nonempty_values: bool,

    #[arg(short, long)]
    fail_fast: bool,

    #[arg(last = true)]
    explicit_input: Option<String>,
}

fn main() {
    let args = Args::parse();

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

    let env: HashMap<String, String> = std::env::vars().collect();

    let restrictions = Restrictions {
        require_explicit_values: args.require_explicit_values,
        require_any_values: args.require_any_values,
        require_nonempty_values: args.require_nonempty_values,
    };

    match process(
        &input_data,
        &env,
        restrictions,
        args.no_digit,
        args.fail_fast,
    ) {
        Ok(result) => {
            if let Some(output_file) = args.output {
                let mut file = File::create(&output_file).unwrap_or_else(|e| {
                    eprintln!("Error creating output file {}: {}", output_file, e);
                    std::process::exit(1);
                });
                file.write_all(result.as_bytes()).unwrap_or_else(|e| {
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
