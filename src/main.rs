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

Uso:
    mcmojave-cursor-unified [OPÇÕES]

Opções:
    -o, --output <DIR>   Diretório de saída para o tema compilado (padrão: dist/McMojave)
    -a, --assets <DIR>   Diretório contendo os SVGs originais (padrão: assets/svg)
    -c, --cache <DIR>    Diretório de cache para PNGs intermediários (padrão: target/png-cache)
    --clean              Remove o diretório de saída e de cache antes de compilar
    -h, --help           Exibe esta mensagem de ajuda
    -v, --version        Exibe a versão do compilador
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
                    eprintln!("Erro: Opção '--output' requer um argumento de diretório.");
                    return ExitCode::FAILURE;
                }
            }
            "-a" | "--assets" => {
                if i + 1 < args.len() {
                    assets_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Erro: Opção '--assets' requer um argumento de diretório.");
                    return ExitCode::FAILURE;
                }
            }
            "-c" | "--cache" => {
                if i + 1 < args.len() {
                    cache_dir = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Erro: Opção '--cache' requer um argumento de diretório.");
                    return ExitCode::FAILURE;
                }
            }
            unknown => {
                eprintln!("Erro: Argumento desconhecido '{unknown}'. Use '--help' para instruções.");
                return ExitCode::FAILURE;
            }
        }
    }

    if clean_before {
        println!("🧹 Limpando diretórios antigos...");
        let _ = fs::remove_dir_all(&output_dir);
        let _ = fs::remove_dir_all(&cache_dir);
    }

    if !assets_dir.exists() {
        eprintln!(
            "Erro: Diretório de assets SVG não encontrado em '{}'",
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
            eprintln!("Falha na compilação: {err}");
            ExitCode::FAILURE
        }
    }
}
