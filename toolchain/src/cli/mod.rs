use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;
use std::{env, fs};

use crate::aot::{build_default_project, run_default_project};
use crate::compiler::compile;
use crate::project::load_project;

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
        Commands::Toolchain { command } => match command {
            ToolchainCommands::Install { dev } => cmd_toolchain_install(dev),
            ToolchainCommands::List => cmd_toolchain_list(),
            ToolchainCommands::Path => cmd_toolchain_path(),
        },
    }
}

fn cmd_new(name: &str) {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        eprintln!("error: directory '{}' already exists", name);
        process::exit(1);
    }

    let src_dir = project_dir.join("src");
    if let Err(e) = fs::create_dir_all(&src_dir) {
        eprintln!("error: failed to create project directory: {}", e);
        process::exit(1);
    }

    let manifest_content = format!(
        "name = \"{}\"\nversion = \"0.1.0\"\nentry = \"src/main.kira\"\n",
        name
    );
    let manifest_path = project_dir.join("kira.project");
    if let Err(e) = fs::write(&manifest_path, manifest_content) {
        eprintln!("error: failed to write kira.project: {}", e);
        process::exit(1);
    }

    let main_content = "func main() {\n    printIn(\"Hello, Kira!\");\n}\n";
    let main_path = src_dir.join("main.kira");
    if let Err(e) = fs::write(&main_path, main_content) {
        eprintln!("error: failed to write src/main.kira: {}", e);
        process::exit(1);
    }

    println!("  Created {}/", name);
}

fn cmd_build() {
    let project_root = find_project_root();
    let out_root = PathBuf::from("out");

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Compiling {} v{}", project.manifest.name, project.manifest.version);

    let start = std::time::Instant::now();
    match build_default_project(&project_root, &out_root) {
        Ok(binary) => {
            let elapsed = start.elapsed();
            println!("  Finished in {:.1}s → {}", elapsed.as_secs_f64(), binary.display());
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_run() {
    let project_root = find_project_root();
    let out_root = PathBuf::from("out");

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Compiling {} v{}", project.manifest.name, project.manifest.version);

    let start = std::time::Instant::now();
    match build_default_project(&project_root, &out_root) {
        Ok(binary) => {
            let elapsed = start.elapsed();
            println!("  Finished in {:.1}s → {}", elapsed.as_secs_f64(), binary.display());
            println!("  Running {}", binary.display());
            println!();
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }

    match run_default_project(&project_root, &out_root) {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_check() {
    let project_root = find_project_root();

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Checking {} v{}", project.manifest.name, project.manifest.version);

    match compile(&project.program) {
        Ok(_) => {
            println!("  ✓ No errors found");
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_clean() {
    let out_dir = PathBuf::from("out");

    if !out_dir.exists() {
        println!("  Nothing to clean");
        return;
    }

    match fs::remove_dir_all(&out_dir) {
        Ok(_) => println!("  Removed out/"),
        Err(e) => {
            eprintln!("error: failed to remove out/: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_version() {
    println!("kira {}", VERSION);
}

fn get_toolchain_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Kira/toolchains"))
            .unwrap_or_else(|| PathBuf::from("~/.kira/toolchains"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .map(|d| d.join("Kira\\toolchains"))
            .unwrap_or_else(|| PathBuf::from("%LOCALAPPDATA%\\Kira\\toolchains"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::home_dir()
            .map(|h| h.join(".kira/toolchains"))
            .unwrap_or_else(|| PathBuf::from("~/.kira/toolchains"))
    }
}

fn cmd_toolchain_install(dev: bool) {
    if dev {
        println!("  Building development toolchain...");
        
        // Find the toolchain source directory
        let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("kira"));
        let toolchain_src = current_exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("toolchain"))
            .filter(|p| p.exists())
            .or_else(|| {
                // Try relative to current directory
                let cwd = env::current_dir().ok()?;
                let candidate = cwd.join("toolchain");
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            });

        let toolchain_src = match toolchain_src {
            Some(dir) => dir,
            None => {
                eprintln!("error: could not find toolchain source directory");
                eprintln!("  Make sure you're running this from the Kira repository root");
                process::exit(1);
            }
        };

        println!("  Found toolchain at: {}", toolchain_src.display());

        // Build in release mode
        let status = process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&toolchain_src)
            .status();

        match status {
            Ok(status) if status.success() => {
                let built_binary = toolchain_src.join("target/release/toolchain");
                if !built_binary.exists() {
                    eprintln!("error: built binary not found at {}", built_binary.display());
                    process::exit(1);
                }

                // Install to toolchain directory
                let toolchain_dir = get_toolchain_dir();
                let version_dir = toolchain_dir.join(format!("dev-{}", VERSION));
                
                if let Err(e) = fs::create_dir_all(&version_dir) {
                    eprintln!("error: failed to create toolchain directory: {}", e);
                    process::exit(1);
                }

                let install_path = version_dir.join("kira");

                // Copy the binary
                if let Err(e) = fs::copy(&built_binary, &install_path) {
                    eprintln!("error: failed to install binary: {}", e);
                    process::exit(1);
                }

                // Make it executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(&install_path) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&install_path, perms);
                    }
                }

                // Create a symlink to the active version
                let active_link = toolchain_dir.join("active");
                let _ = fs::remove_file(&active_link); // Remove old symlink if exists
                
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    if let Err(e) = symlink(&version_dir, &active_link) {
                        eprintln!("warning: failed to create active symlink: {}", e);
                    }
                }
                
                #[cfg(windows)]
                {
                    use std::os::windows::fs::symlink_dir;
                    if let Err(e) = symlink_dir(&version_dir, &active_link) {
                        eprintln!("warning: failed to create active symlink: {}", e);
                    }
                }

                println!("  ✓ Development toolchain installed to: {}", version_dir.display());
                println!("  Version: dev-{}", VERSION);
                println!();
                println!("To add Kira to your PATH, run:");
                println!("  kira toolchain path");
            }
            Ok(_) => {
                eprintln!("error: cargo build failed");
                process::exit(1);
            }
            Err(e) => {
                eprintln!("error: failed to run cargo: {}", e);
                eprintln!("  Make sure cargo is installed and in your PATH");
                process::exit(1);
            }
        }
    } else {
        eprintln!("error: toolchain distribution server is not currently available");
        eprintln!();
        eprintln!("To build and install a development version locally, use:");
        eprintln!("  kira toolchain install --dev");
        process::exit(1);
    }
}

fn cmd_toolchain_list() {
    let toolchain_dir = get_toolchain_dir();
    
    if !toolchain_dir.exists() {
        println!("No toolchains installed.");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira toolchain install --dev");
        return;
    }

    let active_link = toolchain_dir.join("active");
    let active_target = fs::read_link(&active_link).ok();

    println!("Installed toolchains:");
    println!();

    let mut found_any = false;
    if let Ok(entries) = fs::read_dir(&toolchain_dir) {
        let mut versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name() != "active")
            .collect();
        
        versions.sort_by_key(|e| e.file_name());

        for entry in versions {
            found_any = true;
            let version_name = entry.file_name();
            let is_active = active_target
                .as_ref()
                .and_then(|t| t.file_name())
                .map(|n| n == version_name)
                .unwrap_or(false);

            if is_active {
                println!("  {} (active)", version_name.to_string_lossy());
            } else {
                println!("  {}", version_name.to_string_lossy());
            }
        }
    }

    if !found_any {
        println!("  (none)");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira toolchain install --dev");
    }
}

fn cmd_toolchain_path() {
    let toolchain_dir = get_toolchain_dir();
    let bin_dir = toolchain_dir.join("active");

    if !bin_dir.exists() {
        eprintln!("error: no active toolchain found");
        eprintln!();
        eprintln!("Install a toolchain first:");
        eprintln!("  kira toolchain install --dev");
        process::exit(1);
    }

    println!("Add the following to your shell configuration:");
    println!();

    #[cfg(unix)]
    {
        let shell = env::var("SHELL").unwrap_or_else(|_| String::from("bash"));
        
        if shell.contains("fish") {
            println!("  set -gx PATH {} $PATH", bin_dir.display());
            println!();
            println!("For fish shell, add to ~/.config/fish/config.fish:");
            println!("  set -gx PATH {} $PATH", bin_dir.display());
        } else if shell.contains("zsh") {
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
            println!();
            println!("For zsh, add to ~/.zshrc:");
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
        } else {
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
            println!();
            println!("For bash, add to ~/.bashrc or ~/.bash_profile:");
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
        }
    }

    #[cfg(windows)]
    {
        println!("  setx PATH \"%PATH%;{}\"", bin_dir.display());
        println!();
        println!("Or add it manually through System Properties > Environment Variables");
    }
}

fn find_project_root() -> PathBuf {
    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    let mut dir = current.as_path();
    loop {
        let manifest = dir.join("kira.project");
        if manifest.exists() {
            return dir.to_path_buf();
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                eprintln!("error: no kira.project found in current directory or any parent directory");
                process::exit(1);
            }
        }
    }
}
