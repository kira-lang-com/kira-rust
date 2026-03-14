mod commands;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use commands::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "kira")]
#[command(about = "The Kira programming language toolchain", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Kira project
    New { name: String },
    /// Compile the project to a native binary
    Build,
    /// Build and run the project immediately
    Run,
    /// Type-check the project without compiling
    Check,
    /// Remove build artifacts
    Clean,
    /// Print Kira version
    Version,
    /// Manage Kira toolchain installations
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommands,
    },
    /// Package a library for distribution
    Package {
        /// Output directory for the package
        #[arg(short, long, default_value = "out")]
        output: PathBuf,
    },
    /// Fetch and cache project dependencies
    Fetch,
    /// Add a dependency to the project
    Add {
        /// Dependency name
        name: String,
        /// Dependency version
        #[arg(short, long)]
        version: Option<String>,
        /// Local path to dependency
        #[arg(short, long)]
        path: Option<String>,
        /// Git repository URL
        #[arg(short, long)]
        git: Option<String>,
    },
}

#[derive(Subcommand)]
enum ToolchainCommands {
    /// Install a Kira toolchain version
    Install {
        /// Build a development version locally
        #[arg(long)]
        dev: bool,
    },
    /// List installed toolchain versions
    List,
    /// Add Kira toolchain to PATH
    Path,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Build => cmd_build(),
        Commands::Run => cmd_run(),
        Commands::Check => cmd_check(),
        Commands::Clean => cmd_clean(),
        Commands::Version => cmd_version(),
        Commands::Package { output } => cmd_package(&output),
        Commands::Fetch => cmd_fetch(),
        Commands::Add { name, version, path, git } => cmd_add(&name, version, path, git),
        Commands::Toolchain { command } => match command {
            ToolchainCommands::Install { dev } => cmd_toolchain_install(dev),
            ToolchainCommands::List => cmd_toolchain_list(),
            ToolchainCommands::Path => cmd_toolchain_path(),
        },
    }
}

fn cmd_version() {
    println!("kira {}", VERSION);
}
