use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "Test programs CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a program (or all programs if none specified)
    Build { program: Option<String> },
    /// Deploy a program
    Deploy { program: String },
    /// Get the program ID
    GetId { program: String },
    /// Run a program's test binary
    Test { program: String },
    /// List available programs
    List,
}

fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).parent().unwrap().to_path_buf()
}

fn discover_programs() -> Vec<String> {
    let root = workspace_root();
    let mut programs = Vec::new();

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("simd-") && path.join("Cargo.toml").exists() {
                        programs.push(name.to_string());
                    }
                }
            }
        }
    }

    programs.sort();
    programs
}

fn validate_program(program: &str) -> Result<String, String> {
    let programs = discover_programs();
    if programs.contains(&program.to_string()) {
        Ok(program.to_string())
    } else {
        Err(format!(
            "Unknown program '{}'. Available: {}",
            program,
            programs.join(", ")
        ))
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            for program in discover_programs() {
                println!("{}", program);
            }
        }
        Commands::Build { program } => {
            let programs = match program {
                Some(prog) => vec![validate_program(&prog).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                })],
                None => discover_programs(),
            };
            for prog in programs {
                let manifest_path = workspace_root().join(&prog).join("Cargo.toml");
                let status = Command::new("cargo")
                    .args([
                        "build-sbf",
                        "--manifest-path",
                        manifest_path.to_str().unwrap(),
                    ])
                    .status()
                    .expect("failed to execute cargo build-sbf");
                if !status.success() {
                    std::process::exit(status.code().unwrap_or(1));
                }
            }
        }
        Commands::Deploy { program } => {
            let program = validate_program(&program).unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(1);
            });
            let program_name = program.replace('-', "_");
            let so_path = workspace_root()
                .join("target/deploy")
                .join(format!("{}.so", program_name));
            let status = Command::new("solana")
                .args(["program", "deploy", so_path.to_str().unwrap()])
                .status()
                .expect("failed to execute solana program deploy");
            std::process::exit(status.code().unwrap_or(1));
        }
        Commands::GetId { program } => {
            let program = validate_program(&program).unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(1);
            });
            let keypair_path = workspace_root().join(&program).join("keypair.json");
            let status = Command::new("solana")
                .args(["address", "-k", keypair_path.to_str().unwrap()])
                .status()
                .expect("failed to execute solana address");
            std::process::exit(status.code().unwrap_or(1));
        }
        Commands::Test { program } => {
            let program = validate_program(&program).unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(1);
            });
            let status = Command::new("cargo")
                .args(["run", "-p", &program])
                .status()
                .expect("failed to execute cargo run");
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}
