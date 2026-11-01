use std::path::PathBuf;

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Report, miette};

use crate::{
    analyze, load_project_with_source, resolve,
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
            let project = load_project_with_source(&file).map_err(|error| miette!("{error}"))?;

            if let Err(errors) = analyze(&project.program, &project.source_name, &project.source) {
                for error in errors {
                    eprintln!("{:?}", Report::new(error));
                }
                return Err(miette!("source validation failed"));
            }

            println!(
                "{}: valid ({} top-level items)",
                file.display(),
                project.program.items.len()
            );
        }
        Command::Run { file } => {
            let project = load_project_with_source(&file).map_err(|error| miette!("{error}"))?;
            let model = resolve(&project.program, &project.source_name, &project.source)
                .map_err(|error| miette!("{}", error))?;
            let mut query_count = 0usize;

            for item in &project.program.items {
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
                println!("{}: no #show query directives", file.display());
            }
        }
    }

    Ok(())
}
