// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

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

    println!("🎨 [1/5] Building Hyprcursor archives (.hlc)...");
    build_hyprcursors(paths.assets_svg_dir, &hyprcursors_dir)?;

    println!("⚡ [2/5] Rasterizing multi-resolution PNGs (resvg, native)...");
    rasterize_all_pngs(paths.assets_svg_dir, paths.cache_dir)?;

    println!("📦 [3/5] Generating calibrated XCursor binaries (native Rust)...");
    build_xcursors(paths.cache_dir, &cursors_dir)?;

    println!("🔗 [4/5] Creating XCursor symlinks and aliases...");
    create_xcursor_aliases(&cursors_dir)?;

    println!("📝 [5/5] Writing manifest.hl and index.theme...");
    write_theme_metadata(paths.output_dir)?;

    println!("✨ McMojave built successfully at: {}", paths.output_dir.display());
    Ok(())
}

// ─── Hyprcursor (.hlc archives) ───────────────────────────────────────────────

fn build_hyprcursors(assets_dir: &Path, out_dir: &Path) -> Result<(), AppError> {
    CURSOR_DEFS.par_iter().try_for_each(|cursor| -> Result<(), AppError> {
        let dest_file = out_dir.join(format!("{}.hlc", cursor.name));
        let file = File::create(&dest_file)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // Generate meta.hl
        let mut meta = String::new();
        meta.push_str("resize_algorithm = none\n\n");
        meta.push_str(&format!("hotspot_x = {:.2}\n", cursor.hx));
        meta.push_str(&format!("hotspot_y = {:.2}\n\n", cursor.hy));

        if let Some(frames) = cursor.animated_frames {
            for i in 0..frames {
                // BUG-3 FIX: frame 0 uses the base name (e.g. progress.svg),
                // subsequent frames use the -NN suffix (e.g. progress-01.svg)
                let svg_name = if i == 0 {
                    format!("{}.svg", cursor.name)
                } else {
                    format!("{}-{:02}.svg", cursor.name, i)
                };
                meta.push_str(&format!(
                    "define_size = 24, {}, {}\n",
                    svg_name, cursor.frame_delay_ms
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

        // Add SVG asset(s)
        if let Some(frames) = cursor.animated_frames {
            for i in 0..frames {
                let (src_name, zip_name) = if i == 0 {
                    let n = format!("{}.svg", cursor.name);
                    (n.clone(), n)
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

// ─── SVG Rasterization (resvg, native — replaces rsvg-convert) ───────────────

fn rasterize_svg(svg_data: &[u8], size: u32, output: &Path) -> Result<(), AppError> {
    use resvg::usvg;
    use resvg::tiny_skia::{Pixmap, Transform};

    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &options)
        .map_err(|e| AppError::Custom(format!("SVG parse error for '{}': {e}", output.display())))?;

    let svg_w = tree.size().width();
    let svg_h = tree.size().height();

    let mut pixmap = Pixmap::new(size, size)
        .ok_or_else(|| AppError::Custom(format!("Failed to allocate {size}x{size} pixmap")))?;

    let scale_x = size as f32 / svg_w;
    let scale_y = size as f32 / svg_h;
    let transform = Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap
        .save_png(output)
        .map_err(|e| AppError::Custom(format!("PNG save error '{}': {e}", output.display())))?;

    Ok(())
}

fn rasterize_all_pngs(assets_dir: &Path, cache_dir: &Path) -> Result<(), AppError> {
    for &size in NOMINAL_SIZES {
        let size_dir = cache_dir.join(format!("size_{size}"));
        fs::create_dir_all(&size_dir)?;
    }

    // BUG-4 FIX: only rasterize SVGs that belong to a known cursor definition,
    // not every file in the assets directory.
    let mut tasks: Vec<(Vec<u8>, std::path::PathBuf, u32)> = Vec::new();

    for cursor in CURSOR_DEFS {
        // Collect all SVG names for this cursor (base + animated frames)
        let svg_names: Vec<String> = if let Some(frames) = cursor.animated_frames {
            (0..frames)
                .map(|i| {
                    if i == 0 {
                        format!("{}.svg", cursor.name)
                    } else {
                        format!("{}-{:02}.svg", cursor.name, i)
                    }
                })
                .collect()
        } else {
            vec![format!("{}.svg", cursor.name)]
        };

        for svg_name in &svg_names {
            let src_path = assets_dir.join(svg_name);
            if !src_path.exists() {
                return Err(AppError::Custom(format!(
                    "Missing SVG asset: {}",
                    src_path.display()
                )));
            }

            let svg_mtime = fs::metadata(&src_path)
                .and_then(|m| m.modified())
                .ok();

            let svg_data = fs::read(&src_path)?;

            for &size in NOMINAL_SIZES {
                let png_stem = std::path::Path::new(svg_name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let out_path = cache_dir
                    .join(format!("size_{size}"))
                    .join(format!("{png_stem}.png"));

                // OPT-2: skip if PNG exists AND is newer than the SVG source
                if out_path.exists() {
                    if let Some(svg_mt) = svg_mtime {
                        if let Ok(png_mt) = fs::metadata(&out_path).and_then(|m| m.modified()) {
                            if png_mt >= svg_mt {
                                continue; // cache is fresh
                            }
                        }
                    }
                }

                tasks.push((svg_data.clone(), out_path, size));
            }
        }
    }

    tasks
        .par_iter()
        .try_for_each(|(svg_data, dst, size)| rasterize_svg(svg_data, *size, dst))?;

    Ok(())
}

// ─── XCursor binary generation (native Rust — replaces xcursorgen) ────────────

/// XCursor image chunk type constant.
const XCURSOR_IMAGE_TYPE: u32 = 0xfffd0002;
/// XCursor file magic: "Xcur" in little-endian.
const XCURSOR_MAGIC: u32 = 0x72756358;
/// XCursor image chunk header size (9 u32 fields = 36 bytes).
const XCURSOR_CHUNK_HEADER: u32 = 36;
/// XCursor file header size (4 u32 fields = 16 bytes).
const XCURSOR_FILE_HEADER: u32 = 16;
/// XCursor format version.
const XCURSOR_VERSION: u32 = 0x0001_0000;
/// XCursor image chunk version.
const XCURSOR_IMAGE_VERSION: u32 = 1;

/// Encode a sequence of (size, xhot, yhot, delay_ms, RGBA-pixels) tuples into
/// XCursor binary format and write to `dest`.
///
/// `images` is sorted externally; each entry contains:
///   (nominal_size, xhot, yhot, delay_ms, raw_pixmap_rgba_premul)
fn write_xcursor(
    dest: &Path,
    images: &[(u32, u32, u32, u32, Vec<u8>, u32, u32)], // (nominal, xhot, yhot, delay, rgba, w, h)
) -> Result<(), AppError> {
    let ntoc = images.len() as u32;
    let toc_size = ntoc * 12; // each TOC entry = 3 × u32
    let file_header_size = XCURSOR_FILE_HEADER; // 16

    // Compute chunk offsets
    let mut offsets: Vec<u32> = Vec::with_capacity(images.len());
    let mut pos = file_header_size + toc_size;
    for (_, _, _, _, _pixels, w, h) in images {
        offsets.push(pos);
        let chunk_data_bytes = w * h * 4;
        pos += XCURSOR_CHUNK_HEADER + chunk_data_bytes;
    }

    let mut buf: Vec<u8> = Vec::with_capacity(pos as usize);

    // File header
    buf.extend_from_slice(&XCURSOR_MAGIC.to_le_bytes());
    buf.extend_from_slice(&file_header_size.to_le_bytes());
    buf.extend_from_slice(&XCURSOR_VERSION.to_le_bytes());
    buf.extend_from_slice(&ntoc.to_le_bytes());

    // TOC
    for (i, (nominal, _, _, _, _, _, _)) in images.iter().enumerate() {
        buf.extend_from_slice(&XCURSOR_IMAGE_TYPE.to_le_bytes());
        buf.extend_from_slice(&nominal.to_le_bytes());
        buf.extend_from_slice(&offsets[i].to_le_bytes());
    }

    // Image chunks
    for (nominal, xhot, yhot, delay, pixels, w, h) in images {
        let chunk_data_bytes = w * h * 4;
        buf.extend_from_slice(&XCURSOR_CHUNK_HEADER.to_le_bytes());
        buf.extend_from_slice(&XCURSOR_IMAGE_TYPE.to_le_bytes());
        buf.extend_from_slice(&nominal.to_le_bytes());
        buf.extend_from_slice(&XCURSOR_IMAGE_VERSION.to_le_bytes());
        buf.extend_from_slice(&w.to_le_bytes());
        buf.extend_from_slice(&h.to_le_bytes());
        buf.extend_from_slice(&xhot.to_le_bytes());
        buf.extend_from_slice(&yhot.to_le_bytes());
        buf.extend_from_slice(&delay.to_le_bytes());

        // Convert premultiplied RGBA → XCursor ARGB u32 little-endian
        // tiny-skia pixels: [r_premul, g_premul, b_premul, a] per 4 bytes
        for chunk in pixels.chunks_exact(4) {
            let (rp, gp, bp, a) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32, chunk[3] as u32);
            let (r, g, b) = if a == 0 {
                (0u32, 0u32, 0u32)
            } else {
                (
                    (rp * 255 + a / 2) / a,
                    (gp * 255 + a / 2) / a,
                    (bp * 255 + a / 2) / a,
                )
            };
            let argb: u32 = (a << 24) | (r << 16) | (g << 8) | b;
            buf.extend_from_slice(&argb.to_le_bytes());
        }

        // Sanity: chunk body size = chunk_data_bytes
        let _ = chunk_data_bytes; // suppress unused warning
    }

    fs::write(dest, &buf)?;
    Ok(())
}

fn build_xcursors(cache_dir: &Path, cursors_dir: &Path) -> Result<(), AppError> {
    CURSOR_DEFS.par_iter().try_for_each(|cursor| -> Result<(), AppError> {
        let dest_xcursor = cursors_dir.join(cursor.name);

        // Collect all (nominal_size, xhot, yhot, delay_ms, pixels, w, h) tuples
        let mut images: Vec<(u32, u32, u32, u32, Vec<u8>, u32, u32)> = Vec::new();

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
                    let pixels = read_png_as_rgba(&png_path)?;
                    images.push((size, xhot, yhot, cursor.frame_delay_ms, pixels, size, size));
                }
            } else {
                let png_path = cache_dir
                    .join(format!("size_{size}"))
                    .join(format!("{}.png", cursor.name));
                let pixels = read_png_as_rgba(&png_path)?;
                images.push((size, xhot, yhot, 0, pixels, size, size));
            }
        }

        write_xcursor(&dest_xcursor, &images)
    })?;

    Ok(())
}

/// Read a PNG produced by resvg/tiny-skia and return raw premultiplied RGBA bytes.
/// Since we wrote the PNG ourselves with pixmap.save_png(), we know it's RGBA 8-bit.
fn read_png_as_rgba(path: &Path) -> Result<Vec<u8>, AppError> {
    // tiny-skia's save_png writes standard RGBA PNG; we can read raw bytes from it
    // using the same tiny-skia Pixmap loader for consistency.
    use resvg::tiny_skia::Pixmap;
    let pixmap = Pixmap::load_png(path)
        .map_err(|e| AppError::Custom(format!("PNG load error '{}': {e}", path.display())))?;
    Ok(pixmap.data().to_vec())
}

// ─── XCursor aliases (symlinks) ───────────────────────────────────────────────

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

// ─── Theme metadata ───────────────────────────────────────────────────────────

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
