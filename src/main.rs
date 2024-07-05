use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

fn create_temp_dir(temp_dir: &Path) -> Result<(), String> {
    // Create a temporary directory
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| format!("Failed to remove existing temp dir: {}", e))?;
    }
    fs::create_dir(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    Ok(())
}

fn write_main_rs(temp_dir: &Path, code: &str, toml: &str, github_repo: &str, build_code: &str) -> Result<PathBuf, String> {
    fs::create_dir(temp_dir.join("src")).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let main_rs_path = temp_dir.join("src/main.rs");
    let toml_file_path = temp_dir.join("Cargo.toml");
    let build_file_path = temp_dir.join("src/build.rs");
    let config_file_path = temp_dir.join("launcher.config");

    let mut main_rs_file = File::create(&main_rs_path)
        .map_err(|e| format!("Failed to create main.rs: {}", e))?;
    main_rs_file.write_all(code.as_bytes())

        .map_err(|e| format!("Failed to write to main.rs: {}", e))?;
    let mut config_file = File::create(&config_file_path)
        .map_err(|e| format!("Failed to create Cargo.toml: {}", e))?;
    config_file.write_all(github_repo.as_bytes());

    let mut toml_file = File::create(&toml_file_path)
        .map_err(|e| format!("Failed to create Cargo.toml: {}", e))?;
    toml_file.write_all(toml.as_bytes());

    let mut build_file = File::create(&build_file_path)
        .map_err(|e| format!("Failed to create Cargo.toml: {}", e))?;
    build_file.write_all(build_code.as_bytes());



    Ok(temp_dir.to_path_buf())
}

fn compile_rust_file(file_path: &Path) -> Result<(), String> {
   let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(file_path.join("Cargo.toml"))
        //.current_dir(file_path)
        .output()
        .map_err(|e| format!("Failed to execute cargo: {}", e))?;

    if output.status.success() {
        println!("Build succeeded.");

        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Build failed:\n{}", stderr))
    }
}

fn compile_rust_project(project_path: &Path) -> Result<(), String> {
    let mut child = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(project_path.join("Cargo.toml"))     
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute cargo: {}", e))?;

    let stdout = child.stdout.take().ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "Failed to capture stderr".to_string())?;

    let stdout_reader = io::BufReader::new(stdout);
    let stderr_reader = io::BufReader::new(stderr);

    let stdout_thread = std::thread::spawn(move || {
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                eprintln!("{}", line);
            }
        }
    });

    let status = child.wait().map_err(|e| format!("Failed to wait on child process: {}", e))?;

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    if status.success() {
        println!("Build succeeded.");
        fs::rename(project_path.join("target/release/launcher.exe"), "./launcher.exe");
        Ok(())
    } else {
        Err("Build failed".to_string())
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let github_repo = args.get(1).expect("GitHub repo missing");
    println!("Creating launcher from {} repo", github_repo);
    // Include the Rust code as a string
    let rust_code = include_str!("./launcher.rs");
    let toml_code = include_str!("./Cargo.toml");
    let build_code = include_str!("./build.rs");
    let temp_dir = PathBuf::from("./temp");
    match create_temp_dir(&temp_dir) {
        Ok(_) => {
            match write_main_rs(&temp_dir, rust_code, toml_code, github_repo, build_code) {
                Ok(main_rs_path) => {
                    match compile_rust_project(&main_rs_path) {
                        Ok(_) => println!("Project compiled successfully."),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                },
                Err(e) => eprintln!("Error writing main.rs: {}", e),
            }

            // Clean up the temporary directory
            if let Err(e) = fs::remove_dir_all(&temp_dir) {
                eprintln!("Failed to remove temporary directory: {}", e);
            }
        }
        Err(e) => eprintln!("Error creating temporary project: {}", e),
    }
}

