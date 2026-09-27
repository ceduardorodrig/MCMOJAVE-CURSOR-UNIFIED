# MCMOJAVE-CURSOR-UNIFIED

[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-informational)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/wayland-Hyprland-blueviolet)](https://hyprland.org)

A dual-spec, unified modern packaging of the **McMojave** macOS-inspired cursor theme for Linux desktops, combining native **Hyprcursor** (vector SVG) and mathematically calibrated **XCursor** (X11 / XWayland) into a single canonical directory with a blazing-fast native Rust compiler (zero system dependencies).

---

## 🎨 Visual Showcase & Cursors

<div align="center">
  <img src="assets/preview/mcmojave-showcase.png" alt="McMojave Cursor Showcase" width="100%">
</div>

### Animated Busy & Progress Cursors

| `wait` (macOS Beachball) | `progress` (Pointer + Beachball) | Description |
| :---: | :---: | :--- |
| <img src="assets/preview/wait.gif" width="48" alt="wait cursor"> | <img src="assets/preview/progress.gif" width="48" alt="progress cursor"> | 24-frame fluid 30ms animated spinning beachball, compiled natively for both Hyprcursor and animated XCursor. |

---

## 🎯 The Problems in Upstream Ports

While McMojave is one of the cleanest cursor designs for Linux, existing packages on GitHub suffer from critical mathematical flaws that break desktop ergonomics:

```mermaid
flowchart TD
    subgraph Upstream Bugs
        A1[Libadoxon mcmojave-hyprcursor] -->|Divided 32px canvas by 24| B1[Hotspot displacement: 10px click offset on pointer hand]
        A2[Vinceliuice mcmojave-cursors] -->|0.75 nominal size factor in .cursor| B2[Steam/XWayland 33% oversize bug]
        A3[Theme Fragmentation] -->|McMojave vs McMojave-cursors| B3[Toolkit desync & mismatched fallback assets]
        A4[GTK settings.ini Silent Override] -->|Wayland: GTK file beats env vars & gsettings| B4[Steam/GTK apps ignore XCURSOR_THEME entirely]
    end

    subgraph mcmojave-cursor-unified Fix
        C[Unified Rust Engine] --> D1[Exact Hotspot Calibration: pointer hx=0.39, hy=0.19]
        C --> D2[1:1 Mathematical Scaling: nominal size == bitmap dimensions]
        C --> D3[Dual-Spec Theme: hyprcursors/ + cursors/ in single McMojave namespace]
        C --> D4[Full Config Guide: GTK settings.ini must be patched explicitly]
    end
```

### 1. The 10-Pixel Pointer Hand Jump (Hotspot Bug)
In upstream Hyprcursor ports, hotspot coordinates were calculated by dividing raw coordinates by 24 instead of the actual 32×32 SVG canvas.
- For `pointer` (pointing hand), the index finger tip is at `(12.5, 6.0)`. Upstream configured `hotspot_x = 0.67` (16 / 24) instead of `0.39` (12.5 / 32).
- **Symptom:** When hovering over links or buttons, the click target visibly jumped ~10 pixels to the right, causing missed clicks and awkward selection behavior.
- **Fix:** Every cursor hotspot was mathematically recalculated and calibrated against the vector geometry.

### 2. The Steam / XWayland 33% Oversize Bug
Upstream XCursor builder scripts mapped 48px bitmaps to nominal size 36, and 32px bitmaps to nominal 24 (a `0.75` multiplier).
- **Symptom:** Whenever hovering over Steam, KeePassXC, Electron apps, or wine/games, the cursor was 33% larger than on native Wayland windows.
- **Fix:** Strict 1:1 nominal-to-pixel scaling (`24, 28, 32, 36, 40, 48, 64`) ensuring seamless visual consistency across Wayland and XWayland.

### 3. Toolkit Namespace Fragmentation
Upstream distributed Hyprcursor under `McMojave` and XCursor under `McMojave-cursors`.
- **Symptom:** Setting `XCURSOR_THEME="McMojave"` in X11/GTK apps loaded system fallback cursors (Adwaita) because `cursors/` was missing from `McMojave/`.
- **Fix:** Both specifications (`hyprcursors/` with `.hlc` archives and `cursors/` with X11 binary cursors + 64 legacy symlinks) reside in a single directory: `/usr/share/icons/McMojave`.

### 4. The GTK `settings.ini` Silent Override (Wayland-Only)
On Wayland sessions, GTK applications (including Steam) resolve the cursor theme by reading `~/.config/gtk-3.0/settings.ini` and `~/.config/gtk-4.0/settings.ini` **directly**, bypassing `XCURSOR_THEME`, `gsettings`, and `~/.icons/default` entirely.
- **Symptom:** Even with `XCURSOR_THEME=McMojave` in the environment, `gsettings` pointing to `McMojave`, and `/usr/share/icons/default` correctly inheriting `McMojave` — Steam and all GTK apps stubbornly render a different cursor (e.g. `Bibata-Modern-Classic`) because it was set in `settings.ini` by a theme manager and never updated.
- **Root cause:** On Wayland there is no XSETTINGS daemon bridge; GTK reads its own INI files as the authoritative source for cursor configuration. Environment variables and gsettings are only consulted as fallbacks when `settings.ini` is absent or does not specify the key.
- **Fix:** Both `gtk-3.0` and `gtk-4.0` `settings.ini` must explicitly declare:
  ```ini
  gtk-cursor-theme-name=McMojave
  gtk-cursor-theme-size=36
  ```
  See the [Configuration Guide → GTK section](#2-gtk-2-3-4--gnome--gsettings) below. After editing, restart the affected application — no logout required.

---

## 🧵 The Thread

This repository was born out of an everyday desktop frustration: switching between native Wayland applications (running fluidly on Hyprland + Noctalia) and legacy XWayland / Steam games on Arch Linux / CachyOS, only to experience jarring cursor size mismatches and inaccurate click hotspots.

Tracing the issue through compositor logs, XSETTINGS daemons, GTK configuration files, and upstream source code revealed four distinct bugs that had persisted across the Linux cursor ecosystem:
1. An index finger hotspot offset in Hyprcursor vector metadata (`0.67` instead of `0.39`) caused by dividing coordinates by 24 instead of 32.
2. A hardcoded `0.75` nominal sizing multiplier in upstream XCursor scripts, forcing XWayland / Steam to render cursors 33% larger than native Wayland windows.
3. Theme namespace fragmentation between `McMojave` (Hyprcursor) and `McMojave-cursors` (XCursor), causing GTK/X11 apps to fall back to system defaults.
4. On Wayland, GTK `settings.ini` files (`gtk-3.0` / `gtk-4.0`) take authoritative precedence over `XCURSOR_THEME` and `gsettings`, silently overriding the cursor even when all other layers are correctly configured — causing apps like Steam to display a stale cursor set by a theme manager.

Rather than maintaining local patches or brittle shell scripts, this project unifies both specifications into a single canonical `McMojave` theme and provides a pure Rust 2024 compiler to build, calibrate, and package it with mathematical rigor.

---

## ⚡ Native Rust Compiler Engine

Instead of fragile legacy Bash scripts and Python wrappers, this repository includes an ultra-fast, multi-threaded builder written in **pure Rust (2024 edition)** — with **zero system dependencies**:

- **Native SVG Rasterization:** [`resvg`](https://crates.io/crates/resvg) renders SVGs in-process via `tiny-skia`, parallel across all 7 resolutions (`24px` to `64px`) — no `rsvg-convert`, no forks, no `librsvg` required.
- **Native XCursor Encoder:** Pure Rust binary encoder writes the XCursor format directly (ARGB pixel layout, multi-size TOC, animated frame support) — no `xcursorgen`, no `xorg-xcursorgen` required.
- **In-Memory .hlc Archive Generation:** Builds compliant Hyprcursor ZIP archives with clean `meta.hl` descriptors via `rayon` + `zip`.
- **mtime-aware Cache:** Re-rasterizes only SVGs newer than their cached PNG — incremental rebuilds are instant.
- **Sub-Second Build:** Compiles the entire dual-spec theme (47 base cursors, 64 symlinks, 7 resolutions) in under **1 second**.

---

## 🛠️ Installation

### Build & Install from Source

**Requirements:** `rust` (Cargo, 2024 edition, Rust 1.85+) — nothing else.

```bash
git clone https://github.com/ceduardorodrig/MCMOJAVE-CURSOR-UNIFIED.git
cd MCMOJAVE-CURSOR-UNIFIED

# Build + install to /usr/share/icons/McMojave/ + apply to all config layers:
cargo run --release -- --install
```

That's it. `--install` handles everything in one command:
1. Builds the theme (`hyprcursors/` + calibrated `cursors/`)
2. Copies to `/usr/share/icons/McMojave/` (via `sudo`)
3. Applies McMojave to all toolkit layers (GTK 3/4, gsettings, Qt, Wayland env, etc.)

**Other useful commands:**
```bash
# Re-apply all config layers without rebuilding (e.g. after a theme manager reset):
./target/release/mcmojave-cursor-unified --apply-only

# Build only (output to dist/McMojave):
cargo run --release

# Rebuild from scratch:
cargo run --release -- --clean
```

---

## ⚙️ Configuration Guide

> **TL;DR:** `cargo run --release -- --install` configures everything automatically.
> The sections below are for reference or manual overrides only.

`--install` (and `--apply-only`) automatically writes McMojave to **all** of these layers:

| Layer | File / Command |
|---|---|
| Wayland session env | `~/.config/uwsm/env` (`HYPRCURSOR_THEME`, `XCURSOR_THEME`) |
| GTK 3 | `~/.config/gtk-3.0/settings.ini` |
| GTK 4 | `~/.config/gtk-4.0/settings.ini` |
| GTK 2 | `~/.gtkrc-2.0` |
| GSettings | `org.gnome.desktop.interface cursor-theme` |
| xsettingsd | `~/.config/xsettingsd/xsettingsd.conf` |
| X11 fallback | `~/.icons/default/index.theme` |
| System fallback | `/usr/share/icons/default/index.theme` (via sudo) |
| Qt 5/6 | `~/.config/qt5ct/qt5ct.conf`, `~/.config/qt6ct/qt6ct.conf` |

---

### Manual Configuration (reference)

If you prefer to configure layers individually:

#### Hyprland / Wayland session (`~/.config/uwsm/env` or hyprland.conf):
```bash
export HYPRCURSOR_THEME="McMojave"
export HYPRCURSOR_SIZE=36
export XCURSOR_THEME="McMojave"
export XCURSOR_SIZE=36
```

#### GTK 3 & 4 (`~/.config/gtk-{3,4}.0/settings.ini`):

> **⚠️ Wayland Priority Warning:** On Wayland, GTK reads `settings.ini` as the **authoritative source**, overriding `XCURSOR_THEME` and even `gsettings`. If a theme manager (nwg-look, GNOME Tweaks, etc.) previously set a different cursor here, it silently wins. **Always verify `settings.ini` explicitly** — or just run `--apply-only` after any theme manager change.

```ini
[Settings]
gtk-cursor-theme-name=McMojave
gtk-cursor-theme-size=36
```

#### GSettings (fallback — only if `settings.ini` does not set the key):
```bash
gsettings set org.gnome.desktop.interface cursor-theme 'McMojave'
gsettings set org.gnome.desktop.interface cursor-size 36
```

#### Qt 5 & 6 (`~/.config/qt6ct/qt6ct.conf`):
```ini
[Appearance]
cursor=McMojave
cursor_size=36
```

#### Steam / Wine / stubborn X11 apps (xsettingsd):
```ini
# ~/.config/xsettingsd/xsettingsd.conf
Gtk/CursorThemeName "McMojave"
Gtk/CursorThemeSize 36
```

#### Xresources (legacy X11):
```text
Xcursor.theme: McMojave
Xcursor.size: 36
```
Apply with `xrdb -merge ~/.Xresources`.

---

## 📄 License

- Graphics and assets derived from [McMojave-cursors](https://github.com/vinceliuice/McMojave-cursors) by **Vinceliuice** (GPL-3.0).
- Compiler tooling and unified packaging: Copyright (C) 2026 **Carlos Eduardo Rodrigues** under **GPL-3.0-or-later** ([LICENSE](LICENSE)).

---

<div align="center">

> **Yes... This is a Vibe Coded project**
>
> Governed by 🤖 **StenioSentinel** (our Rust-based AI Governance Sentinel) with **Carlos Eduardo Rodrigues** ([@ceduardorodrig](https://github.com/ceduardorodrig)).

</div>
