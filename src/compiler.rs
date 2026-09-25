// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::config::{CURSOR_DEFS, NOMINAL_SIZES, XCURSOR_ALIASES};
use crate::error::AppError;

pub struct BuildPaths<'a> {
    pub assets_svg_dir: &'a Path,
    pub cache_dir: &'a Path,
    pub output_dir: &'a Path,
}

pub fn build_all(paths: &BuildPaths<'_>) -> Result<(), AppError> {
    let hyprcursors_dir = paths.output_dir.join("hyprcursors");
    let cursors_dir = paths.output_dir.join("cursors");

    fs::create_dir_all(&hyprcursors_dir)?;
    fs::create_dir_all(&cursors_dir)?;
    fs::create_dir_all(paths.cache_dir)?;

    println!("🎨 [1/5] Compilando Hyprcursor archives (.hlc)...");
    build_hyprcursors(paths.assets_svg_dir, &hyprcursors_dir)?;

    println!("⚡ [2/5] Rasterizando PNGs multi-resolução com rsvg-convert...");
    rasterize_all_pngs(paths.assets_svg_dir, paths.cache_dir)?;

    println!("📦 [3/5] Gerando cursores XCursor calibrados com xcursorgen...");
    build_xcursors(paths.cache_dir, &cursors_dir)?;

    println!("🔗 [4/5] Criando symlinks e aliases XCursor...");
    create_xcursor_aliases(&cursors_dir)?;

    println!("📝 [5/5] Escrevendo manifest.hl e index.theme...");
    write_theme_metadata(paths.output_dir)?;

    println!("✨ McMojave compilado com sucesso em: {}", paths.output_dir.display());
    Ok(())
}

fn build_hyprcursors(assets_dir: &Path, out_dir: &Path) -> Result<(), AppError> {
    CURSOR_DEFS.par_iter().try_for_each(|cursor| -> Result<(), AppError> {
        let dest_file = out_dir.join(format!("{}.hlc", cursor.name));
        let file = File::create(&dest_file)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // Gera meta.hl
        let mut meta = String::new();
        meta.push_str("resize_algorithm = none\n\n");
        meta.push_str(&format!("hotspot_x = {:.2}\n", cursor.hx));
        meta.push_str(&format!("hotspot_y = {:.2}\n\n", cursor.hy));

        if let Some(frames) = cursor.animated_frames {
            for i in 0..frames {
                meta.push_str(&format!(
                    "define_size = 24, {}-{:02}.svg, {}\n",
                    cursor.name, i, cursor.frame_delay_ms
                ));
            }
        } else {
            meta.push_str(&format!("define_size = 0,{}.svg\n", cursor.name));
        }

        if !cursor.overrides.is_empty() {
            meta.push('\n');
            for ov in cursor.overrides {
                meta.push_str(&format!("define_override = {ov}\n"));
            }
        }

        zip.start_file("meta.hl", options)?;
        zip.write_all(meta.as_bytes())?;

        // Adiciona arquivos SVG
        if let Some(frames) = cursor.animated_frames {
            for i in 0..frames {
                let (src_name, zip_name) = if i == 0 {
                    (format!("{}.svg", cursor.name), format!("{}-00.svg", cursor.name))
                } else {
                    let n = format!("{}-{:02}.svg", cursor.name, i);
                    (n.clone(), n)
                };

                let src_path = assets_dir.join(&src_name);
                let mut content = Vec::new();
                File::open(&src_path)?.read_to_end(&mut content)?;

                zip.start_file(&zip_name, options)?;
                zip.write_all(&content)?;
            }
        } else {
            let svg_name = format!("{}.svg", cursor.name);
            let src_path = assets_dir.join(&svg_name);
            let mut content = Vec::new();
            File::open(&src_path)?.read_to_end(&mut content)?;

            zip.start_file(&svg_name, options)?;
            zip.write_all(&content)?;
        }

        zip.finish()?;
        Ok(())
    })?;

    Ok(())
}

fn rasterize_all_pngs(assets_dir: &Path, cache_dir: &Path) -> Result<(), AppError> {
    for &size in NOMINAL_SIZES {
        let size_dir = cache_dir.join(format!("size_{size}"));
        fs::create_dir_all(&size_dir)?;
    }

    let mut tasks = Vec::new();
    for entry in fs::read_dir(assets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("svg") {
            if let Some(file_name) = path.file_name() {
                for &size in NOMINAL_SIZES {
                    let out_path = cache_dir
                        .join(format!("size_{size}"))
                        .join(file_name)
                        .with_extension("png");
                    tasks.push((path.clone(), out_path, size));
                }
            }
        }
    }

    tasks.par_iter().try_for_each(|(src, dst, size)| -> Result<(), AppError> {
        if dst.exists() {
            return Ok(());
        }

        let output = Command::new("rsvg-convert")
            .arg("-w")
            .arg(size.to_string())
            .arg("-h")
            .arg(size.to_string())
            .arg("-f")
            .arg("png")
            .arg("-o")
            .arg(dst)
            .arg(src)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(AppError::ProcessFailed {
                command: format!("rsvg-convert {} -> {}", src.display(), dst.display()),
                code: output.status.code(),
                stderr,
            });
        }

        Ok(())
    })?;

    Ok(())
}

fn build_xcursors(cache_dir: &Path, cursors_dir: &Path) -> Result<(), AppError> {
    CURSOR_DEFS.par_iter().try_for_each(|cursor| -> Result<(), AppError> {
        let config_path = cache_dir.join(format!("{}.cursor", cursor.name));
        let dest_xcursor = cursors_dir.join(cursor.name);

        let mut config_content = String::new();

        for &size in NOMINAL_SIZES {
            let max_coord = size.saturating_sub(1);
            let xhot = ((cursor.hx * size as f32).round() as u32).min(max_coord);
            let yhot = ((cursor.hy * size as f32).round() as u32).min(max_coord);

            if let Some(frames) = cursor.animated_frames {
                for i in 0..frames {
                    let frame_file = if i == 0 {
                        format!("{}.png", cursor.name)
                    } else {
                        format!("{}-{:02}.png", cursor.name, i)
                    };
                    let png_path = cache_dir.join(format!("size_{size}")).join(&frame_file);
                    config_content.push_str(&format!(
                        "{} {} {} {} {}\n",
                        size,
                        xhot,
                        yhot,
                        png_path.display(),
                        cursor.frame_delay_ms
                    ));
                }
            } else {
                let png_path = cache_dir
                    .join(format!("size_{size}"))
                    .join(format!("{}.png", cursor.name));
                config_content.push_str(&format!(
                    "{} {} {} {}\n",
                    size,
                    xhot,
                    yhot,
                    png_path.display()
                ));
            }
        }

        fs::write(&config_path, config_content)?;

        let output = Command::new("xcursorgen")
            .arg(&config_path)
            .arg(&dest_xcursor)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(AppError::ProcessFailed {
                command: format!("xcursorgen {} {}", config_path.display(), dest_xcursor.display()),
                code: output.status.code(),
                stderr,
            });
        }

        Ok(())
    })?;

    Ok(())
}

fn create_xcursor_aliases(cursors_dir: &Path) -> Result<(), AppError> {
    for &(alias, target) in XCURSOR_ALIASES {
        let link_path = cursors_dir.join(alias);
        if link_path.exists() || fs::symlink_metadata(&link_path).is_ok() {
            let _ = fs::remove_file(&link_path);
        }
        std::os::unix::fs::symlink(target, &link_path)?;
    }
    Ok(())
}

fn write_theme_metadata(output_dir: &Path) -> Result<(), AppError> {
    let manifest_path = output_dir.join("manifest.hl");
    let index_theme_path = output_dir.join("index.theme");

    let manifest_content = r#"name = McMojave
description = Unified McMojave cursor theme (Hyprcursor vector + 1:1 calibrated XCursor)
version = 1.0.0
cursors_directory = hyprcursors
"#;

    let index_theme_content = r#"[Icon Theme]
Name=McMojave
Comment=Unified McMojave cursor theme (Hyprcursor vector + 1:1 calibrated XCursor)
Inherits="hicolor"
"#;

    fs::write(manifest_path, manifest_content)?;
    fs::write(index_theme_path, index_theme_content)?;

    Ok(())
}
