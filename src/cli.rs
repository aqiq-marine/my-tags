use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Report, miette};

use crate::{
    analyze, load_project, resolve,
    ast::Item,
    query_dsl,
};

#[derive(Debug, Parser)]
#[command(name = "my-tags", version, about = "Validate and query my-tags source files")]
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
    /// Evaluate and display all #show query directives in a source file.
    Run {
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
            let program = load_project(&file).map_err(|error| miette!("{error}"))?;

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
        Command::Run { file } => {
            let source = fs::read_to_string(&file).into_diagnostic()?;
            let source_name = file.display().to_string();
            let program = parse(&source_name, &source).map_err(Report::new)?;
            let model = resolve(&program, &source_name, &source).map_err(|error| miette!("{}", error))?;
            let mut query_count = 0usize;

            for item in &program.items {
                if let Item::ShowQuery(query) = item {
                    query_count += 1;
                    let mut ids: Vec<_> = query_dsl::evaluate(&model, &query.expression)
                        .map_err(|error| miette!("{}", error))?
                        .into_iter()
                        .collect();
                    ids.sort_unstable();
                    println!("Query {} result ({} instances):", query_count, ids.len());
                    for id in ids {
                        if let Some(instance) = model.instance_by_id(id) {
                            println!("  {}", instance.name);
                        }
                    }
                }
            }

            if query_count == 0 {
                println!("{}: no #show query directives", source_name);
            }
        }
    }

    Ok(())
}
