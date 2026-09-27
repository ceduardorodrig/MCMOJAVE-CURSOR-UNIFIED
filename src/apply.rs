// Copyright (C) 2026 Carlos Eduardo Rodrigues
// SPDX-License-Identifier: GPL-3.0-or-later

//! `--apply`: propagates McMojave cursor theme to every toolkit layer on the system.
//!
//! Layers handled (user-space, no sudo required unless noted):
//!   1.  gsettings (org.gnome.desktop.interface)
//!   2.  ~/.config/gtk-3.0/settings.ini
//!   3.  ~/.config/gtk-4.0/settings.ini
//!   4.  ~/.gtkrc-2.0
//!   5.  ~/.config/xsettingsd/xsettingsd.conf
//!   6.  ~/.icons/default/index.theme
//!   7.  /usr/share/icons/default/index.theme  (requires root / sudo)
//!   8.  ~/.config/qt5ct/qt5ct.conf            (if qt5ct is installed)
//!   9.  ~/.config/qt6ct/qt6ct.conf            (if qt6ct is installed)
//!  10.  ~/.config/uwsm/env                     (Hyprland / UWSM session env)

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const THEME: &str = "McMojave";
const SIZE: u32 = 36;

// ─── public entry point ───────────────────────────────────────────────────────

pub fn apply_all() -> Result<(), String> {
    let home = home_dir()?;
    let mut applied = 0usize;
    let mut skipped = 0usize;

    println!("🎯 McMojave Cursor Unified — applying theme to all toolkit layers");
    println!("   Theme : {THEME}");
    println!("   Size  : {SIZE}");
    println!();

    // 1. gsettings
    match apply_gsettings() {
        Ok(true) => { println!("  ✅ gsettings"); applied += 1; }
        Ok(false) => { println!("  ⏭️  gsettings — gsettings binary not found, skipping"); skipped += 1; }
        Err(e) => println!("  ⚠️  gsettings — {e}"),
    }

    // 2. GTK 3
    let gtk3 = home.join(".config/gtk-3.0/settings.ini");
    match apply_gtk_ini(&gtk3) {
        Ok(()) => { println!("  ✅ gtk-3.0/settings.ini"); applied += 1; }
        Err(e) => println!("  ⚠️  gtk-3.0/settings.ini — {e}"),
    }

    // 3. GTK 4
    let gtk4 = home.join(".config/gtk-4.0/settings.ini");
    match apply_gtk_ini(&gtk4) {
        Ok(()) => { println!("  ✅ gtk-4.0/settings.ini"); applied += 1; }
        Err(e) => println!("  ⚠️  gtk-4.0/settings.ini — {e}"),
    }

    // 4. GTK 2
    let gtkrc2 = home.join(".gtkrc-2.0");
    match apply_gtkrc2(&gtkrc2) {
        Ok(()) => { println!("  ✅ .gtkrc-2.0 (GTK2)"); applied += 1; }
        Err(e) => println!("  ⚠️  .gtkrc-2.0 — {e}"),
    }

    // 5. xsettingsd
    let xsettingsd = home.join(".config/xsettingsd/xsettingsd.conf");
    match apply_xsettingsd(&xsettingsd) {
        Ok(true) => { println!("  ✅ xsettingsd.conf"); applied += 1; }
        Ok(false) => { println!("  ⏭️  xsettingsd.conf — file not found, skipping"); skipped += 1; }
        Err(e) => println!("  ⚠️  xsettingsd.conf — {e}"),
    }

    // 6. ~/.icons/default
    let icons_default = home.join(".icons/default/index.theme");
    match apply_icons_default(&icons_default) {
        Ok(()) => { println!("  ✅ ~/.icons/default/index.theme"); applied += 1; }
        Err(e) => println!("  ⚠️  ~/.icons/default — {e}"),
    }

    // 7. /usr/share/icons/default (needs root)
    let sys_default = PathBuf::from("/usr/share/icons/default/index.theme");
    match apply_sys_default(&sys_default) {
        Ok(true) => { println!("  ✅ /usr/share/icons/default/index.theme"); applied += 1; }
        Ok(false) => { println!("  ⏭️  /usr/share/icons/default — sudo unavailable, skipping"); skipped += 1; }
        Err(e) => println!("  ⚠️  /usr/share/icons/default — {e}"),
    }

    // 8. qt5ct (optional)
    let qt5ct = home.join(".config/qt5ct/qt5ct.conf");
    if qt5ct.exists() {
        match apply_qtct(&qt5ct) {
            Ok(()) => { println!("  ✅ qt5ct.conf"); applied += 1; }
            Err(e) => println!("  ⚠️  qt5ct.conf — {e}"),
        }
    } else {
        println!("  ⏭️  qt5ct.conf — not installed, skipping");
        skipped += 1;
    }

    // 9. qt6ct (optional)
    let qt6ct = home.join(".config/qt6ct/qt6ct.conf");
    if qt6ct.exists() {
        match apply_qtct(&qt6ct) {
            Ok(()) => { println!("  ✅ qt6ct.conf"); applied += 1; }
            Err(e) => println!("  ⚠️  qt6ct.conf — {e}"),
        }
    } else {
        println!("  ⏭️  qt6ct.conf — not installed, skipping");
        skipped += 1;
    }

    // 10. uwsm/env
    let uwsm_env = home.join(".config/uwsm/env");
    if uwsm_env.exists() {
        match apply_uwsm_env(&uwsm_env) {
            Ok(()) => { println!("  ✅ uwsm/env"); applied += 1; }
            Err(e) => println!("  ⚠️  uwsm/env — {e}"),
        }
    } else {
        println!("  ⏭️  uwsm/env — not found, skipping");
        skipped += 1;
    }

    println!();
    println!("  Applied: {applied} | Skipped: {skipped}");
    println!();
    println!("  ✨ Done. Restart open applications to pick up the new cursor.");
    println!("     (Steam, Electron apps and GTK apps need a full restart)");

    Ok(())
}

// ─── layer implementations ────────────────────────────────────────────────────

/// Layer 1 — gsettings
fn apply_gsettings() -> Result<bool, String> {
    if which("gsettings").is_none() {
        return Ok(false);
    }
    run_cmd("gsettings", &[
        "set", "org.gnome.desktop.interface", "cursor-theme", THEME,
    ])?;
    run_cmd("gsettings", &[
        "set", "org.gnome.desktop.interface", "cursor-size", &SIZE.to_string(),
    ])?;
    Ok(true)
}

/// Layers 2 & 3 — GTK 3 / GTK 4 INI files.
///
/// Handles three scenarios:
///   a) Key already present → in-place line replacement
///   b) `[Settings]` section exists but key absent → insert after section header
///   c) File doesn't exist → create minimal file from scratch
fn apply_gtk_ini(path: &Path) -> Result<(), String> {
    if !path.exists() {
        // Create parent dirs + minimal file
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = format!(
            "[Settings]\ngtk-cursor-theme-name={THEME}\ngtk-cursor-theme-size={SIZE}\n"
        );
        fs::write(path, content).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = raw.lines().map(str::to_owned).collect();

    let mut found_name = false;
    let mut found_size = false;

    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("gtk-cursor-theme-name") && trimmed.contains('=') {
            *line = format!("gtk-cursor-theme-name={THEME}");
            found_name = true;
        } else if trimmed.starts_with("gtk-cursor-theme-size") && trimmed.contains('=') {
            *line = format!("gtk-cursor-theme-size={SIZE}");
            found_size = true;
        }
    }

    // If keys were absent, insert them after [Settings]
    if !found_name || !found_size {
        let mut insert_after = None;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "[Settings]" {
                insert_after = Some(i);
                break;
            }
        }
        let pos = insert_after.unwrap_or(lines.len());
        if !found_size {
            lines.insert(pos + 1, format!("gtk-cursor-theme-size={SIZE}"));
        }
        if !found_name {
            lines.insert(pos + 1, format!("gtk-cursor-theme-name={THEME}"));
        }
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

/// Layer 4 — ~/.gtkrc-2.0 (GTK2 key=value format, no sections)
fn apply_gtkrc2(path: &Path) -> Result<(), String> {
    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = existing.lines().map(str::to_owned).collect();
    let mut found_name = false;
    let mut found_size = false;

    for line in &mut lines {
        let key = line.split('=').next().unwrap_or("").trim();
        if key == "gtk-cursor-theme-name" {
            *line = format!("gtk-cursor-theme-name=\"{THEME}\"");
            found_name = true;
        } else if key == "gtk-cursor-theme-size" {
            *line = format!("gtk-cursor-theme-size={SIZE}");
            found_size = true;
        }
    }

    if !found_name {
        lines.push(format!("gtk-cursor-theme-name=\"{THEME}\""));
    }
    if !found_size {
        lines.push(format!("gtk-cursor-theme-size={SIZE}"));
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

/// Layer 5 — xsettingsd.conf (only touch if file exists; daemon-managed)
fn apply_xsettingsd(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = raw.lines().map(str::to_owned).collect();
    let mut found_name = false;
    let mut found_size = false;

    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Gtk/CursorThemeName") {
            *line = format!("Gtk/CursorThemeName \"{THEME}\"");
            found_name = true;
        } else if trimmed.starts_with("Gtk/CursorThemeSize") {
            *line = format!("Gtk/CursorThemeSize {SIZE}");
            found_size = true;
        }
    }

    if !found_name {
        lines.push(format!("Gtk/CursorThemeName \"{THEME}\""));
    }
    if !found_size {
        lines.push(format!("Gtk/CursorThemeSize {SIZE}"));
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Layer 6 — ~/.icons/default/index.theme
fn apply_icons_default(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = format!(
        "[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={THEME}\n"
    );
    fs::write(path, content).map_err(|e| e.to_string())
}

/// Layer 7 — /usr/share/icons/default/index.theme (needs root)
fn apply_sys_default(path: &Path) -> Result<bool, String> {
    if which("sudo").is_none() {
        return Ok(false);
    }

    let content = format!(
        "[Icon Theme]\nName=default ({THEME})\n\
         Comment=Fallback cursor theme → {THEME} (Steam/chroot resolvem via 'default')\n\
         Inherits={THEME}\n"
    );

    // Write to a temp file, then sudo mv into place
    let tmp = PathBuf::from(format!("/tmp/mcmojave-default-{}.theme", std::process::id()));
    fs::write(&tmp, &content).map_err(|e| e.to_string())?;

    // Ensure parent exists
    let parent = path.parent().unwrap_or(Path::new("/usr/share/icons/default"));
    run_cmd("sudo", &["mkdir", "-p", &parent.to_string_lossy()])?;
    run_cmd("sudo", &["cp", &tmp.to_string_lossy(), &path.to_string_lossy()])?;

    let _ = fs::remove_file(&tmp);
    Ok(true)
}

/// Layers 8 & 9 — qt5ct / qt6ct INI (same format, different paths)
fn apply_qtct(path: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = raw.lines().map(str::to_owned).collect();
    let mut found_cursor = false;
    let mut in_appearance = false;
    let mut appearance_idx = None;

    for (i, line) in lines.iter_mut().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[Appearance]" {
            in_appearance = true;
            appearance_idx = Some(i);
            continue;
        }
        if trimmed.starts_with('[') {
            in_appearance = false;
        }
        if in_appearance {
            let key = trimmed.split('=').next().unwrap_or("").trim();
            if key == "cursor" {
                *line = format!("cursor={THEME}");
                found_cursor = true;
            }
        }
    }

    if !found_cursor {
        // Insert after [Appearance] section header, or append
        let pos = appearance_idx.unwrap_or(lines.len());
        lines.insert(pos + 1, format!("cursor={THEME}"));
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

/// Layer 10 — ~/.config/uwsm/env (shell export format)
fn apply_uwsm_env(path: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = raw.lines().map(str::to_owned).collect();

    let mut found_xcursor = false;
    let mut found_xcursor_size = false;
    let mut found_hyprcursor = false;
    let mut found_hyprcursor_size = false;

    for line in &mut lines {
        let trimmed = line.trim_start();
        // Match both `export VAR=value` and bare `VAR=value`
        let var = trimmed
            .trim_start_matches("export ")
            .split('=')
            .next()
            .unwrap_or("")
            .trim();
        match var {
            "XCURSOR_THEME" => {
                *line = format!("export XCURSOR_THEME=\"{THEME}\"");
                found_xcursor = true;
            }
            "XCURSOR_SIZE" => {
                *line = format!("export XCURSOR_SIZE={SIZE}");
                found_xcursor_size = true;
            }
            "HYPRCURSOR_THEME" => {
                *line = format!("export HYPRCURSOR_THEME=\"{THEME}\"");
                found_hyprcursor = true;
            }
            "HYPRCURSOR_SIZE" => {
                *line = format!("export HYPRCURSOR_SIZE={SIZE}");
                found_hyprcursor_size = true;
            }
            _ => {}
        }
    }

    if !found_xcursor {
        lines.push(format!("export XCURSOR_THEME=\"{THEME}\""));
    }
    if !found_xcursor_size {
        lines.push(format!("export XCURSOR_SIZE={SIZE}"));
    }
    if !found_hyprcursor {
        lines.push(format!("export HYPRCURSOR_THEME=\"{THEME}\""));
    }
    if !found_hyprcursor_size {
        lines.push(format!("export HYPRCURSOR_SIZE={SIZE}"));
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "HOME environment variable not set".to_owned())
}

fn which(bin: &str) -> Option<()> {
    Command::new("which")
        .arg(bin)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|_| ())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run `{program}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` exited with {status}", args.join(" ")))
    }
}

// suppress unused import warning — BufRead brought in for potential future use
#[allow(dead_code)]
fn _use_bufread<R: BufRead>(_: R) {}
#[allow(dead_code)]
fn _use_write<W: Write>(_: W) {}
#[allow(dead_code)]
fn _use_io(_: io::Error) {}
