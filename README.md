# Gemini Float ✦

Una ventana flotante que encapsula exclusivamente Google Gemini. Sin barras de dirección, sin pestañas, sin distracciones. Solo tú y la IA.

**Entorno objetivo:** Linux (CachyOS / GNOME / Wayland)  
**Stack:** Tauri 2 (Rust + WebKitGTK)

---

## ⚡ Características

- **Ventana sin bordes**, siempre encima de las demás aplicaciones
- **Arranque en la sombra** — al iniciar, no muestra ninguna ventana
- **Toggle instantáneo** — `Ctrl+Alt+Space` para mostrar/ocultar
- **Auto-ocultamiento** — al hacer clic fuera, la ventana desaparece
- **Instancia única** — nunca se duplica el proceso
- **System Tray** — icono con menú contextual (Mostrar/Ocultar, Salir)

---

## 📦 Prerequisitos

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js y pnpm (CachyOS / Arch)
sudo pacman -S nodejs npm
npm install -g pnpm

# Dependencias del sistema para Tauri 2 (ya incluidas en CachyOS normalmente)
sudo pacman -S webkit2gtk-4.1 gtk4 base-devel
```

---

## 🚀 Desarrollo

```bash
# Instalar dependencias JS
pnpm install

# Ejecutar en modo desarrollo
pnpm tauri dev
```

---

## 🏗️ Build de Producción

```bash
pnpm tauri build
```

El binario se genera en `src-tauri/target/release/gemini-float`.

---

## ⌨️ Configuración del Atajo en Wayland (GNOME)

En Wayland puro, las aplicaciones no pueden capturar atajos globales de teclado. Para que `Ctrl+Alt+Space` funcione:

### Opción 1: GNOME Settings (Recomendado)

1. Abre **Settings → Keyboard → Custom Shortcuts**
2. Haz clic en **Add Shortcut**
3. Configura:
   - **Nombre:** `Gemini Float Toggle`
   - **Comando:** `/ruta/al/binario/gemini-float --toggle`
   - **Atajo:** `Ctrl+Alt+Space`

### Opción 2: CLI directo

```bash
# Si el proceso ya está corriendo, esto togglea la ventana:
gemini-float --toggle

# Si no está corriendo, esto lo inicia:
gemini-float
```

> **Nota:** En sesiones X11/XWayland, el atajo global funciona directamente sin configuración adicional.

---

## 🔄 Autostart con GNOME

Crea el archivo `~/.config/autostart/gemini-float.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=Gemini Float
Comment=Floating Gemini AI wrapper
Exec=/ruta/al/binario/gemini-float
Icon=gemini-float
Hidden=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
StartupNotify=false
```

---

## 📁 Estructura del Proyecto

```
gemini-float/
├── src/                      # Frontend (loading screen fallback)
│   ├── index.html
│   └── styles.css
├── src-tauri/                # Backend Rust
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   └── lib.rs            # Core: toggle, tray, shortcuts, focus
│   ├── capabilities/
│   │   └── default.json      # Permisos mínimos
│   ├── icons/
│   │   └── icon.png          # Icono de la app y tray
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## 📄 Licencia

MIT
