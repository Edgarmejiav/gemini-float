#!/usr/bin/env bash
#
# install.sh — Script de instalación local para gemini-float
# Compatible con: Arch Linux / CachyOS / Manjaro / Debian / Ubuntu / Genérico
#
# Uso:
#   ./install.sh          → Compilar e instalar en el sistema nativo
#   ./install.sh --help   → Mostrar ayuda
#
set -euo pipefail

# ─── Colores ────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# ─── Ayuda ───────────────────────────────────────────────────────────────────
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    echo -e "${BOLD}gemini-float — Script de instalación${NC}"
    echo ""
    echo "Uso: ./install.sh"
    echo ""
    echo "En sistemas Arch/CachyOS usa 'makepkg -si' (estándar nativo)."
    echo "En otros sistemas realiza compilación directa e instalación en /usr/local."
    exit 0
fi

# ─── Directorio raíz del proyecto ────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ─── PATH de herramientas de usuario (nvm / rustup) ──────────────────────────
# Se exporta antes de cualquier comprobación para que cargo/pnpm sean detectables
# incluso cuando se invoca el script fuera de una sesión interactiva de shell.
export PATH="$HOME/.local/share/nvm/default/bin:$HOME/.cargo/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
# Soporte explícito para nvm con versión fija como fallback
[[ -d "/home/lega/.local/share/nvm/v24.16.0/bin" ]] && \
    export PATH="/home/lega/.local/share/nvm/v24.16.0/bin:$PATH"

echo -e "${BLUE}${BOLD}=== 🚀 Gemini Float — Instalación ===${NC}"
echo ""

# ─── RAMA 1: Arch Linux / CachyOS (vía makepkg — Estándar nativo) ────────────
if command -v pacman &>/dev/null && command -v makepkg &>/dev/null; then
    echo -e "${GREEN}[Arch Linux / CachyOS detectado]${NC}"
    echo -e "${YELLOW}Usando makepkg (estándar nativo de Arch)...${NC}"

    # Nos movemos al directorio de empaquetado local para que makepkg
    # no interfiera con la carpeta /src/ del frontend de Tauri.
    cd "$SCRIPT_DIR/packaging"

    # Limpiar artefactos de compilaciones previas (pkg/, src/, *.pkg.tar.zst)
    rm -rf pkg src ./*.pkg.tar.zst

    # makepkg -si: compila (build), empaqueta (package) e instala con pacman
    # --noconfirm: sin preguntas interactivas de pacman
    makepkg -si --noconfirm

    cd "$SCRIPT_DIR"

# ─── RAMA 2: Debian / Ubuntu (vía .deb de Tauri) ─────────────────────────────
elif command -v dpkg &>/dev/null && command -v apt-get &>/dev/null; then
    echo -e "${GREEN}[Debian / Ubuntu detectado]${NC}"

    _check_tool() {
        if ! command -v "$1" &>/dev/null; then
            echo -e "${RED}Error: '$1' no encontrado en PATH.${NC}"
            echo "Instálalo con: ${YELLOW}${2}${NC}"
            exit 1
        fi
    }
    _check_tool pnpm  "npm install -g pnpm"
    _check_tool cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

    echo -e "${BLUE}📦 Instalando dependencias Node.js...${NC}"
    pnpm install --frozen-lockfile

    echo -e "${BLUE}🏗️  Compilando con Tauri (genera .deb)...${NC}"
    pnpm tauri build

    DEB_FILE=$(find src-tauri/target/release/bundle/deb/ \
        \( -name "gemini-float*.deb" -o -name "Gemini_Float*.deb" \) \
        2>/dev/null | head -n1)

    if [[ -f "$DEB_FILE" ]]; then
        echo -e "${BLUE}⚙️  Instalando paquete .deb...${NC}"
        sudo dpkg -i "$DEB_FILE" || sudo apt-get install -f -y
    else
        echo -e "${RED}Error: No se encontró el .deb en src-tauri/target/release/bundle/deb/${NC}"
        exit 1
    fi

# ─── RAMA 3: Genérico / Fedora / openSUSE (instalación manual FHS) ───────────
else
    echo -e "${YELLOW}[Sistema genérico detectado]${NC}"
    echo -e "${YELLOW}Instalación manual en rutas FHS (/usr/local/)...${NC}"

    _check_tool() {
        if ! command -v "$1" &>/dev/null; then
            echo -e "${RED}Error: '$1' no encontrado en PATH.${NC}"
            echo "Instálalo con: ${YELLOW}${2}${NC}"
            exit 1
        fi
    }
    _check_tool pnpm  "npm install -g pnpm"
    _check_tool cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

    echo -e "${BLUE}📦 Instalando dependencias Node.js...${NC}"
    pnpm install --frozen-lockfile

    echo -e "${BLUE}🏗️  Compilando binario con Tauri...${NC}"
    pnpm tauri build --no-bundle

    echo -e "${BLUE}⚙️  Copiando archivos a rutas FHS (/usr/local/)...${NC}"
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

    # Actualizar caches del sistema
    command -v gtk-update-icon-cache &>/dev/null && \
        sudo gtk-update-icon-cache -q -t -f /usr/local/share/icons/hicolor || true
    command -v update-desktop-database &>/dev/null && \
        sudo update-desktop-database -q /usr/local/share/applications || true
fi

# ─── Configurar atajo de teclado en GNOME (como usuario actual) ───────────────
if command -v gsettings &>/dev/null; then
    echo ""
    echo -e "${BLUE}⌨️  Configurando atajo Ctrl+Alt+Space en GNOME...${NC}"

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

    echo -e "${GREEN}✓ Atajo Ctrl+Alt+Space registrado en GNOME.${NC}"
fi

# ─── Resumen final ────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}=== 🎉 ¡Gemini Float instalado correctamente! ===${NC}"
echo ""
echo -e "  ${BOLD}1.${NC} Menú de GNOME → busca ${YELLOW}Gemini Float${NC}"
echo -e "  ${BOLD}2.${NC} Terminal:  ${YELLOW}gemini-float${NC}"
echo -e "  ${BOLD}3.${NC} Atajo:     ${YELLOW}Ctrl + Alt + Space${NC}"
echo ""
