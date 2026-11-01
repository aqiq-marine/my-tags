use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Report, miette};

use crate::{analyze, parse};

#[derive(Debug, Parser)]
#[command(name = "my-tags", version, about = "Validate my-tags source files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and semantically validate a source file.
    Check {
        /// Path to a .mt source file.
        file: PathBuf,
    },
}

pub fn run() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Check { file } => {
            let source = fs::read_to_string(&file).into_diagnostic()?;
            let source_name = file.display().to_string();
            let program = parse(&source_name, &source).map_err(Report::new)?;

            if let Err(errors) = analyze(&program, &source_name, &source) {
                for error in errors {
                    eprintln!("{:?}", Report::new(error));
                }
                return Err(miette!("source validation failed"));
            }

            println!(
                "{}: valid ({} top-level items)",
                source_name,
                program.items.len()
            );
        }
    }

    Ok(())
}
