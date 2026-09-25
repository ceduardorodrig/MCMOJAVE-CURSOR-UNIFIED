// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

mod compiler;
mod config;
mod error;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use compiler::{build_all, BuildPaths};

fn print_help() {
    println!(
        r#"McMojave Cursor Unified Builder

Usage:
    mcmojave-cursor-unified [OPTIONS]

Options:
    -o, --output <DIR>   Output directory for the compiled theme (default: dist/McMojave)
    -a, --assets <DIR>   Directory containing the original SVGs (default: assets/svg)
    -c, --cache <DIR>    Cache directory for intermediate PNGs (default: target/png-cache)
    --clean              Remove the output and cache directories before building
    -h, --help           Show this help message
    -v, --version        Show the compiler version
"#
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("dist/McMojave");
    let mut assets_dir = PathBuf::from("assets/svg");
    let mut cache_dir = PathBuf::from("target/png-cache");
    let mut clean_before = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-v" | "--version" => {
                println!("mcmojave-cursor-unified v{}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--clean" => {
                clean_before = true;
                i += 1;
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: option '--output' requires a directory argument.");
                    return ExitCode::FAILURE;
                }
            }
            "-a" | "--assets" => {
                if i + 1 < args.len() {
                    assets_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: option '--assets' requires a directory argument.");
                    return ExitCode::FAILURE;
                }
            }
            "-c" | "--cache" => {
                if i + 1 < args.len() {
                    cache_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: option '--cache' requires a directory argument.");
                    return ExitCode::FAILURE;
                }
            }
            unknown => {
                eprintln!("Error: unknown argument '{unknown}'. Use '--help' for usage.");
                return ExitCode::FAILURE;
            }
        }
    }

    if clean_before {
        println!("🧹 Cleaning old directories...");
        let _ = fs::remove_dir_all(&output_dir);
        let _ = fs::remove_dir_all(&cache_dir);
    }

    if !assets_dir.exists() {
        eprintln!(
            "Error: SVG assets directory not found at '{}'",
            assets_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let paths = BuildPaths {
        assets_svg_dir: &assets_dir,
        cache_dir: &cache_dir,
        output_dir: &output_dir,
    };

    match build_all(&paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Build failed: {err}");
            ExitCode::FAILURE
        }
    }
}
