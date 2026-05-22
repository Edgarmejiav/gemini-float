use tauri::{
    Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Embedded tray icon (PNG bytes compiled into the binary).
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/32x32.png");

/// Toggle the main window visibility.
/// If visible → hide. If hidden → show and focus.
fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);

        if visible && focused {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    let is_dev = cfg!(debug_assertions);

    builder
        // ── Plugin: Single Instance ──────────────────────────────────
        // When a second instance is launched (e.g. `gemini-float --toggle`),
        // instead of spawning a new process, this callback fires on the
        // *existing* instance and toggles the window.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // Any invocation of a second instance toggles the window.
            // This is the key mechanism for the GNOME custom shortcut on Wayland:
            //   Command: /path/to/gemini-float --toggle
            let _ = args; // args available if needed for future flags
            toggle_window(app);
        }))
        // ── Plugin: Global Shortcut (works on X11/XWayland) ──────────
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // ── Setup ────────────────────────────────────────────────────
        .setup(move |app| {
            // ── Create the main window ───────────────────────────────
            // Loads Gemini directly. No local frontend is shown.
            // We inject a tiny transparent 'drag region' bar at the very top of
            // the webpage (24px height) so the borderless window can be dragged
            // from its top edge. The cursor changes to 'move' when hovering.
            let drag_script = r#"
                (function() {
                    const dragDiv = document.createElement('div');
                    dragDiv.setAttribute('data-tauri-drag-region', '');
                    dragDiv.id = 'tauri-drag-bar';
                    dragDiv.style.position = 'fixed';
                    dragDiv.style.top = '0';
                    dragDiv.style.left = '0';
                    dragDiv.style.width = '100%';
                    dragDiv.style.height = '24px';
                    dragDiv.style.zIndex = '999999';
                    dragDiv.style.cursor = 'move';
                    dragDiv.style.background = 'rgba(0, 0, 0, 0.001)';
                    dragDiv.style.display = 'flex';
                    dragDiv.style.justifyContent = 'center';
                    dragDiv.style.alignItems = 'center';
                    dragDiv.style.pointerEvents = 'auto';

                    // Indicador visual premium
                    const handle = document.createElement('div');
                    handle.setAttribute('data-tauri-drag-region', '');
                    handle.style.width = '40px';
                    handle.style.height = '4px';
                    handle.style.backgroundColor = 'rgba(128, 128, 128, 0.4)';
                    handle.style.borderRadius = '2px';
                    handle.style.pointerEvents = 'none'; // Deja que el evento pase al contenedor principal

                    dragDiv.appendChild(handle);
                    
                    const inject = () => {
                        if (!document.getElementById('tauri-drag-bar')) {
                            document.body.appendChild(dragDiv);
                        }
                    };

                    if (document.body) {
                        inject();
                    } else {
                        document.addEventListener('DOMContentLoaded', inject);
                    }
                })();
            "#;

            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://gemini.google.com".parse().unwrap()),
            )
            .title("Gemini Float")
            .inner_size(900.0, 700.0)
            .center()
            .resizable(true)         // Habilitar explícitamente redimensionar
            .shadow(true)            // Activar sombra nativa para habilitar bordes interactivos
            .decorations(false)      // No title bar, no borders
            .always_on_top(true)     // Float above everything
            .skip_taskbar(!is_dev)   // In dev, show in taskbar so you can find the window
            .visible(is_dev)         // In dev, start visible; in release, keep shadow mode
            .initialization_script(drag_script)
            .build()?;

            // ── Register Ctrl+Alt+Space global shortcut ──────────────
            // This works on X11 and XWayland sessions. On pure Wayland,
            // the user configures a GNOME custom shortcut that runs
            // `gemini-float --toggle` (handled by single-instance plugin).
            let shortcut: Shortcut = "ctrl+alt+space".parse().unwrap();
            let app_handle = app.handle().clone();

            if let Err(err) = app.global_shortcut().on_shortcut(
                shortcut,
                move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(&app_handle);
                    }
                },
            ) {
                eprintln!("Advertencia: No se pudo registrar el atajo global Ctrl+Alt+Space (puede que ya esté en uso por el sistema): {}", err);
            }

            // ── System Tray ──────────────────────────────────────────
            let show_item = MenuItemBuilder::with_id("show", "Mostrar/Ocultar")
                .build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Salir")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_icon = Image::from_bytes(TRAY_ICON_PNG)?;

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("Gemini Float — Ctrl+Alt+Space")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => toggle_window(app),
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        // ── Hide window when it loses focus ──────────────────────────
        .on_window_event(|window, event| {
            if !cfg!(debug_assertions) {
                if let WindowEvent::Focused(false) = event {
                    if window.label() == "main" {
                        let _ = window.hide();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
