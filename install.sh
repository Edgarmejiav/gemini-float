#!/usr/bin/env bash
#
# install.sh - Local installer script for gemini-float
# Compatible with: Arch Linux / CachyOS / Manjaro / Debian / Ubuntu / Generic
#
# Usage:
#   ./install.sh          -> Build and install on the native system
#   ./install.sh --help   -> Show help
#
set -euo pipefail

# --- Colors -----------------------------------------------------------------
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# --- Help -------------------------------------------------------------------
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    echo -e "${BOLD}gemini-float - Installer script${NC}"
    echo ""
    echo "Usage: ./install.sh"
    echo ""
    echo "On Arch/CachyOS systems it uses 'makepkg -si' (native standard)."
    echo "On other systems it performs a direct build and installs to /usr/local."
    exit 0
fi

# --- Project root directory --------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- User tool PATH (nvm / rustup) ------------------------------------------
# Export before checks so cargo/pnpm can be discovered
# even when the script is run outside an interactive shell session.
export PATH="$HOME/.local/share/nvm/default/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
# Explicit support for pinned nvm version as fallback
[[ -d "/home/lega/.local/share/nvm/v24.16.0/bin" ]] && \
    export PATH="/home/lega/.local/share/nvm/v24.16.0/bin:$PATH"

echo -e "${BLUE}${BOLD}=== 🚀 Gemini Float - Installation ===${NC}"
echo ""

# --- BRANCH 1: Arch Linux / CachyOS (via makepkg - Native standard) ---------
if command -v pacman &>/dev/null && command -v makepkg &>/dev/null; then
    echo -e "${GREEN}[Arch Linux / CachyOS detected]${NC}"
    echo -e "${YELLOW}Using makepkg (Arch native standard)...${NC}"

    # Move to the local packaging directory so makepkg
    # does not interfere with Tauri frontend /src/.
    cd "$SCRIPT_DIR/packaging"

    # Clean artifacts from previous builds (pkg/, src/, *.pkg.tar.zst)
    rm -rf pkg src ./*.pkg.tar.zst

    # makepkg -si: build, package, and install via pacman
    # --noconfirm: no interactive pacman prompts
    makepkg -si --noconfirm

    cd "$SCRIPT_DIR"

# --- BRANCH 2: Debian / Ubuntu (via Tauri .deb) -----------------------------
elif command -v dpkg &>/dev/null && command -v apt-get &>/dev/null; then
    echo -e "${GREEN}[Debian / Ubuntu detected]${NC}"

    _check_tool() {
        if ! command -v "$1" &>/dev/null; then
            echo -e "${RED}Error: '$1' not found in PATH.${NC}"
            echo "Install it with: ${YELLOW}${2}${NC}"
            exit 1
        fi
    }
    _check_tool pnpm  "npm install -g pnpm"
    _check_tool cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

    echo -e "${BLUE}📦 Installing Node.js dependencies...${NC}"
    pnpm install --frozen-lockfile

    echo -e "${BLUE}🏗️  Building with Tauri (generates .deb)...${NC}"
    pnpm tauri build

    DEB_FILE=$(find src-tauri/target/release/bundle/deb/ \
        \( -name "gemini-float*.deb" -o -name "Gemini_Float*.deb" \) \
        2>/dev/null | head -n1)

    if [[ -f "$DEB_FILE" ]]; then
        echo -e "${BLUE}⚙️  Installing .deb package...${NC}"
        sudo dpkg -i "$DEB_FILE" || sudo apt-get install -f -y
    else
        echo -e "${RED}Error: .deb not found in src-tauri/target/release/bundle/deb/${NC}"
        exit 1
    fi

# --- BRANCH 3: Generic / Fedora / openSUSE (manual FHS install) -------------
else
    echo -e "${YELLOW}[Generic system detected]${NC}"
    echo -e "${YELLOW}Manual install to FHS paths (/usr/local/)...${NC}"

    _check_tool() {
        if ! command -v "$1" &>/dev/null; then
            echo -e "${RED}Error: '$1' not found in PATH.${NC}"
            echo "Install it with: ${YELLOW}${2}${NC}"
            exit 1
        fi
    }
    _check_tool pnpm  "npm install -g pnpm"
    _check_tool cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

    echo -e "${BLUE}📦 Installing Node.js dependencies...${NC}"
    pnpm install --frozen-lockfile

    echo -e "${BLUE}🏗️  Building binary with Tauri...${NC}"
    pnpm tauri build --no-bundle

    echo -e "${BLUE}⚙️  Copying files to FHS paths (/usr/local/)...${NC}"
    sudo install -Dm755 "src-tauri/target/release/gemini-float" \
        "/usr/local/bin/gemini-float"
    sudo install -Dm644 "src-tauri/icons/icon.png" \
        "/usr/local/share/icons/hicolor/256x256/apps/gemini-float.png"
    sudo install -Dm644 "src-tauri/icons/128x128.png" \
        "/usr/local/share/icons/hicolor/128x128/apps/gemini-float.png"
    sudo install -Dm644 "src-tauri/icons/32x32.png" \
        "/usr/local/share/icons/hicolor/32x32/apps/gemini-float.png"
    sudo install -Dm644 "gemini-float.desktop" \
        "/usr/local/share/applications/gemini-float.desktop"
    sudo install -Dm644 "LICENSE" \
        "/usr/local/share/licenses/gemini-float/LICENSE"

    # Update system caches
    command -v gtk-update-icon-cache &>/dev/null && \
        sudo gtk-update-icon-cache -q -t -f /usr/local/share/icons/hicolor || true
    command -v update-desktop-database &>/dev/null && \
        sudo update-desktop-database -q /usr/local/share/applications || true
fi

# --- Configure GNOME keyboard shortcut (current user) -----------------------
if command -v gsettings &>/dev/null; then
    echo ""
    echo -e "${BLUE}⌨️  Configuring Ctrl+Space shortcut in GNOME...${NC}"

    KEYPATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom-gemini-float/"

    gsettings set \
        org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:"$KEYPATH" \
        name    "Gemini Float Toggle"
    gsettings set \
        org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:"$KEYPATH" \
        command "gemini-float --toggle"
    gsettings set \
        org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:"$KEYPATH" \
        binding "<Control><Alt>space"

    CURRENT=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)

    if [[ "$CURRENT" == "@as []" || "$CURRENT" == "[]" ]]; then
        gsettings set org.gnome.settings-daemon.plugins.media-keys \
            custom-keybindings "['$KEYPATH']"
    elif [[ "$CURRENT" != *"$KEYPATH"* ]]; then
        CLEANED="${CURRENT#[}"
        CLEANED="${CLEANED%]}"
        gsettings set org.gnome.settings-daemon.plugins.media-keys \
            custom-keybindings "[${CLEANED}, '$KEYPATH']"
    fi

    echo -e "${GREEN}✓ Ctrl+Space shortcut registered in GNOME.${NC}"
fi

# --- Final summary -----------------------------------------------------------
echo ""
echo -e "${GREEN}${BOLD}=== 🎉 Gemini Float installed successfully! ===${NC}"
echo ""
echo -e "  ${BOLD}1.${NC} GNOME menu -> search for ${YELLOW}Gemini Float${NC}"
echo -e "  ${BOLD}2.${NC} Terminal:   ${YELLOW}gemini-float${NC}"
echo -e "  ${BOLD}3.${NC} Shortcut:   ${YELLOW}Ctrl + Alt + Space${NC}"
echo ""
