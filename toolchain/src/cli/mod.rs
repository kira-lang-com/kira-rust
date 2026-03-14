mod commands;
mod utils;

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

use commands::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "kira")]
#[command(about = "The Kira programming language toolchain", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Kira project
    New { name: String },
    /// Install the Kira toolchain into `~/kira/toolchains/<mode>/<version>/`
    Install {
        /// Install a release toolchain
        #[arg(long, conflicts_with = "dev")]
        release: bool,
        /// Install a development toolchain
        #[arg(long, conflicts_with = "release")]
        dev: bool,
    },
    /// Compile the project to a native binary
    Build {
        /// Build a native dynamic library (`.dylib`/`.so`/`.dll`) instead of an executable
        #[arg(long, conflicts_with = "bin")]
        lib: bool,
        /// Build a native executable (default)
        #[arg(long, conflicts_with = "lib")]
        bin: bool,
    },
    /// Generate FFI bindings for linked headers
    Ffi,
    /// Build and run the project immediately
    Run,
    /// Run a pre-compiled Kira module (internal use)
    #[command(hide = true)]
    RunModule {
        /// Path to the compiled module
        module: PathBuf,
    },
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
        #[arg(short, long, default_value = "target")]
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
    /// Set the current/default toolchain (updates `~/kira/toolchains/current/kira`)
    Use {
        /// Toolchain identifier in the form `dev/<version>` or `release/<version>`
        toolchain: String,
    },
    /// Add Kira toolchain to PATH
    Path,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Treat `kira` with no args as a successful "show help" invocation.
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            println!();
        }
        Some(Commands::New { name }) => cmd_new(&name),
        Some(Commands::Install { release, dev }) => cmd_install(release, dev),
        Some(Commands::Build { lib, bin }) => cmd_build(lib, bin),
        Some(Commands::Run) => cmd_run(),
        Some(Commands::Ffi) => cmd_ffi(),
        Some(Commands::RunModule { module }) => cmd_run_module(&module),
        Some(Commands::Check) => cmd_check(),
        Some(Commands::Clean) => cmd_clean(),
        Some(Commands::Version) => cmd_version(),
        Some(Commands::Package { output }) => cmd_package(&output),
        Some(Commands::Fetch) => cmd_fetch(),
        Some(Commands::Add { name, version, path, git }) => cmd_add(&name, version, path, git),
        Some(Commands::Toolchain { command }) => match command {
            ToolchainCommands::Install { dev } => cmd_toolchain_install(dev),
            ToolchainCommands::List => cmd_toolchain_list(),
            ToolchainCommands::Use { toolchain } => cmd_toolchain_use(&toolchain),
            ToolchainCommands::Path => cmd_toolchain_path(),
        },
    }
}

fn cmd_version() {
    println!("kira {}", VERSION);
}
