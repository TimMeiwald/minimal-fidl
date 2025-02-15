use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod fmt;
/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "git")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Fmt {
        /// Paths to format
        paths: Vec<PathBuf>,
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,
    },
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args);
    match &args.command {
        Commands::Fmt { paths, dry_run } => fmt::minimal_fidl_fmt(paths, *dry_run),
    }
}
