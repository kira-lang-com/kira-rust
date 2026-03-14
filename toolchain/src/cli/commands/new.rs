use std::fs;
use std::path::PathBuf;
use std::process;

pub fn cmd_new(name: &str) {
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
