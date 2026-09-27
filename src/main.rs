mod apply;
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
        r#"McMojave Cursor Unified — Builder & System Configurator

Usage:
    mcmojave-cursor-unified [OPTIONS]

Build options:
    -o, --output <DIR>   Output directory for the compiled theme (default: dist/McMojave)
    -a, --assets <DIR>   Directory containing the original SVGs (default: assets/svg)
    -c, --cache <DIR>    Cache directory for intermediate PNGs (default: target/png-cache)
    --clean              Remove the output and cache directories before building

Apply options:
    --apply              Apply McMojave cursor to ALL toolkit layers on this system,
                         then build the theme as usual
    --apply-only         Apply to all layers WITHOUT rebuilding the theme

General:
    -h, --help           Show this help message
    -v, --version        Show the compiler version

Layers configured by --apply / --apply-only:
    gsettings · gtk-3.0/settings.ini · gtk-4.0/settings.ini · .gtkrc-2.0
    xsettingsd.conf · ~/.icons/default · /usr/share/icons/default (sudo)
    qt5ct.conf (if installed) · qt6ct.conf (if installed) · uwsm/env
"#
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("dist/McMojave");
    let mut assets_dir = PathBuf::from("assets/svg");
    let mut cache_dir = PathBuf::from("target/png-cache");
    let mut clean_before = false;
    let mut do_apply = false;
    let mut apply_only = false;

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
            "--apply" => {
                do_apply = true;
                i += 1;
            }
            "--apply-only" => {
                apply_only = true;
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
    // --apply-only: configure all layers and exit (no build)
    if apply_only {
        return match apply::apply_all() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Apply failed: {e}");
                ExitCode::FAILURE
            }
        };
    }

    // --apply: configure all layers, then fall through to build
    if do_apply {
        if let Err(e) = apply::apply_all() {
            eprintln!("Apply failed: {e}");
            return ExitCode::FAILURE;
        }
        println!();
        println!("🔨 Building theme...");
        println!();
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
