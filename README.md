# Gemini Float ✦

A floating window wrapper focused exclusively on Google Gemini. No address bar, no tabs, no distractions. Just you and the AI.

**Target environment:** Linux (CachyOS / GNOME / Wayland)  
**Stack:** Tauri 2 (Rust + WebKitGTK)

---

## ⚡ Features

- **Borderless window**, always on top of other applications
- **Shadow startup** — starts without showing any window
- **Instant toggle** — `Ctrl+Alt+Space` to show/hide
- **Auto-hide** — click outside and the window disappears
- **Single instance** — never spawns duplicate processes
- **System Tray** — icon with context menu (Show/Hide, Quit)

---

## 📦 Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js and pnpm (CachyOS / Arch)
sudo pacman -S nodejs npm
npm install -g pnpm

# System dependencies for Tauri 2 (usually already available on CachyOS)
sudo pacman -S webkit2gtk-4.1 gtk4 base-devel
```

---

## 🚀 Development

```bash
# Install JS dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

---

## 🏗️ Production Build

```bash
pnpm tauri build
```

The binary is generated at `src-tauri/target/release/gemini-float`.

---

## ⌨️ Wayland Shortcut Setup (GNOME)

On pure Wayland, apps cannot capture global keyboard shortcuts directly. To make `Ctrl+Alt+Space` work:

### Option 1: GNOME Settings (Recommended)

1. Open **Settings → Keyboard → Custom Shortcuts**
2. Click **Add Shortcut**
3. Set:
   - **Name:** `Gemini Float Toggle`
   - **Command:** `/path/to/gemini-float --toggle`
   - **Shortcut:** `Ctrl+Alt+Space`

### Option 2: Direct CLI

```bash
# If the process is already running, this toggles the window:
gemini-float --toggle

# If it is not running, this starts it:
gemini-float
```

> **Note:** In X11/XWayland sessions, the global shortcut works directly without extra setup.

---

## 🔄 GNOME Autostart

Create the file `~/.config/autostart/gemini-float.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=Gemini Float
Comment=Floating Gemini AI wrapper
Exec=/path/to/gemini-float
Icon=gemini-float
Hidden=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
StartupNotify=false
```

---

## 📦 Packaging and the installer script

This project provides both a native Arch packaging flow and a convenience installer script for local
usage and development. Below are the responsibilities of each and when to use them.

- `PKGBUILD` / `makepkg` (Arch/CachyOS):
  - The `PKGBUILD` file is used to build a native Arch package with `makepkg`.
  - Its `package()` function copies the final binary and supporting files into the package layout, for
    example `/usr/bin/gemini-float`, `/usr/share/icons/...`, and `/usr/share/applications/gemini-float.desktop`.
  - The output is a `.pkg.tar.zst` which can be installed with `pacman -U` or uploaded to the AUR.

- `install.sh` (multi-distro helper):
  - `install.sh` automates common local install workflows across distributions:
    - On Arch-like systems it delegates to `makepkg` inside the `packaging/` folder.
    - On Debian/Ubuntu it builds a `.deb` via `pnpm tauri build` and attempts to install it with `dpkg`.
    - On generic systems it builds without bundling and copies files to FHS locations (`/usr/local/bin`,
      `/usr/local/share/icons`, `/usr/local/share/applications`, etc.).
  - Use `install.sh` for quick local installs during development or when you want the script to manage
    build-and-install steps for you.

Notes:
- The repository contains the source `PKGBUILD`, `.install` hooks and `.desktop` file, but it intentionally
  excludes generated artifacts (for example `packaging/pkg/` or any `*.pkg.tar.zst`). Those build outputs are
  added to `.gitignore` and should not be committed.
- If you maintain separate packaging scripts, keep them in `packaging/` and ensure only source files are
  tracked in Git (not compiled packages).

---

## 📁 Project Structure

```
gemini-float/
├── src/                      # Frontend (loading screen fallback)
│   ├── index.html
│   └── styles.css
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   └── lib.rs            # Core: toggle, tray, shortcuts, focus
│   ├── capabilities/
│   │   └── default.json      # Minimal permissions
│   ├── icons/
│   │   └── icon.png          # App and tray icon
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## 📄 License

MIT
