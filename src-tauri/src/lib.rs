use std::sync::{Mutex, OnceLock};
use tauri::{
    Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Icono embebido del tray (PNG compilado en el binario).
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/32x32.png");

/// Última posición conocida de la ventana.
/// Se guarda al ocultar y se restaura al volver a mostrar.
fn last_position() -> &'static Mutex<Option<PhysicalPosition<i32>>> {
    static LAST_POS: OnceLock<Mutex<Option<PhysicalPosition<i32>>>> = OnceLock::new();
    LAST_POS.get_or_init(|| Mutex::new(None))
}

/// Alterna visibilidad de la ventana principal.
/// - Visible + enfocada → guardar posición y ocultar.
/// - Oculta / sin foco  → mostrar y restaurar posición.
fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);

        if visible && focused {
            // Guardar posición actual antes de ocultar
            if let Ok(pos) = window.outer_position() {
                if let Ok(mut last) = last_position().lock() {
                    *last = Some(pos);
                }
            }
            let _ = window.hide();
        } else {
            let _ = window.show();
            // Restaurar última posición conocida
            if let Ok(last) = last_position().lock() {
                if let Some(pos) = *last {
                    let _ = window.set_position(pos);
                }
            }
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let is_dev = cfg!(debug_assertions);

    tauri::Builder::default()
        // ── Plugin: Una sola instancia ───────────────────────────────
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            toggle_window(app);
        }))
        // ── Plugin: Atajo global (X11 / XWayland) ────────────────────
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // ── Setup ────────────────────────────────────────────────────
        .setup(move |app| {
            // Script inyectado en el contexto del webview de Gemini.
            //
            // Estrategia de drag:
            //   mousedown en el header → navega a gemini-float://window/start-drag
            //   Rust intercepta → llama window.start_dragging() → Tauri mueve la ventana
            //   return false cancela la navegación antes de que el webview cambie de página
            //
            // Estrategia de espacio:
            //   El header es position:fixed y NO modifica el padding del body,
            //   por lo que el área útil del webview permanece intacta (100vh).
            let header_script = r#"
(function() {
    if (window.self !== window.top) return;
    if (window.__gf_header__) return;
    window.__gf_header__ = true;

    var header = null;

    function buildHeader() {
        var el = document.createElement('div');
        el.id = 'tauri-custom-header';
        el.style.cssText =
            'position:fixed!important;top:0!important;left:0!important;' +
            'width:100%!important;height:40px!important;' +
            'z-index:2147483647!important;' +
            'background:rgba(13,13,23,0.94)!important;' +
            'backdrop-filter:blur(18px)!important;' +
            '-webkit-backdrop-filter:blur(18px)!important;' +
            'border-bottom:1px solid rgba(255,255,255,0.08)!important;' +
            'display:flex!important;align-items:center!important;' +
            'justify-content:space-between!important;' +
            'padding:0 14px!important;box-sizing:border-box!important;' +
            'font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif!important;' +
            'user-select:none!important;-webkit-user-select:none!important;' +
            'cursor:default!important;';

        /* ── Marca izquierda ─────────────────────────────────────── */
        var brand = document.createElement('div');
        brand.style.cssText =
            'display:flex;align-items:center;gap:9px;pointer-events:none;flex-shrink:0;cursor:move;';

        var orb = document.createElement('div');
        orb.style.cssText =
            'width:11px;height:11px;border-radius:50%;flex-shrink:0;' +
            'background:linear-gradient(135deg,#9b51e0,#3085fe,#70e2ff);' +
            'box-shadow:0 0 9px rgba(112,226,255,0.95);';

        var appName = document.createElement('span');
        appName.textContent = 'Gemini Float';
        appName.style.cssText =
            'color:#f2f2f2;font-size:12px;font-weight:600;letter-spacing:0.65px;';

        brand.appendChild(orb);
        brand.appendChild(appName);
        el.appendChild(brand);

        /* ── Handle central de arrastre ─────────────────────────── */
        var handleWrap = document.createElement('div');
        handleWrap.style.cssText =
            'position:absolute;left:50%;transform:translateX(-50%);' +
            'pointer-events:none;cursor:move;';
        var handle = document.createElement('div');
        handle.style.cssText =
            'width:34px;height:3px;background:rgba(255,255,255,0.13);border-radius:2px;';
        handleWrap.appendChild(handle);
        el.appendChild(handleWrap);

        /* ── Botones de control (derecha) ───────────────────────── */
        var controls = document.createElement('div');
        controls.style.cssText =
            'display:flex;align-items:center;gap:7px;pointer-events:auto;flex-shrink:0;';
        controls.setAttribute('data-gf-controls', '');

        function makeBtn(dim, full, border, label) {
            var b = document.createElement('div');
            b.title = label;
            b.setAttribute('data-gf-controls', ''); // marca para excluir del drag
            b.style.cssText =
                'width:12px;height:12px;border-radius:50%;' +
                'display:flex;align-items:center;justify-content:center;' +
                'cursor:pointer;pointer-events:auto;' +
                'border:1px solid ' + border + ';' +
                'background-color:' + dim + ';' +
                'transition:background-color 0.15s;';
            b._dim  = dim;
            b._full = full;
            return b;
        }

        var minBtn   = makeBtn('rgba(255,189,46,0.35)','rgb(255,189,46)','rgba(255,189,46,0.6)','Minimizar');
        var maxBtn   = makeBtn('rgba(40,200,64,0.35)', 'rgb(40,200,64)', 'rgba(40,200,64,0.6)', 'Maximizar');
        var closeBtn = makeBtn('rgba(255,95,87,0.35)', 'rgb(255,95,87)', 'rgba(255,95,87,0.6)', 'Cerrar');

        controls.addEventListener('mouseenter', function() {
            minBtn.style.backgroundColor   = minBtn._full;
            maxBtn.style.backgroundColor   = maxBtn._full;
            closeBtn.style.backgroundColor = closeBtn._full;
        });
        controls.addEventListener('mouseleave', function() {
            minBtn.style.backgroundColor   = minBtn._dim;
            maxBtn.style.backgroundColor   = maxBtn._dim;
            closeBtn.style.backgroundColor = closeBtn._dim;
        });

        /* Clicks → navegan al esquema personalizado interceptado en Rust */
        minBtn.addEventListener('click',   function(e){ e.stopPropagation(); e.preventDefault(); window.location.href='gemini-float://window/minimize'; });
        maxBtn.addEventListener('click',   function(e){ e.stopPropagation(); e.preventDefault(); window.location.href='gemini-float://window/toggle-maximize'; });
        closeBtn.addEventListener('click', function(e){ e.stopPropagation(); e.preventDefault(); window.location.href='gemini-float://window/close'; });

        controls.appendChild(minBtn);
        controls.appendChild(maxBtn);
        controls.appendChild(closeBtn);
        el.appendChild(controls);

        /* ── Drag nativo via start_dragging() en Rust ───────────── */
        el.addEventListener('mousedown', function(e) {
            if (e.button !== 0) return;
            // Ignorar si el click viene de los botones de control
            var t = e.target;
            while (t && t !== el) {
                if (t.getAttribute && t.getAttribute('data-gf-controls') !== null) return;
                t = t.parentElement;
            }
            e.preventDefault();
            e.stopPropagation();
            // Notificar a Rust para iniciar el drag nativo del SO
            window.location.href = 'gemini-float://window/start-drag';
        });

        return el;
    }

    function inject() {
        if (!document.body) return;

        if (!header) {
            header = buildHeader();
        }

        /* Reinsertar si la SPA de Gemini lo sacó del DOM */
        if (!header.isConnected) {
            document.body.insertBefore(header, document.body.firstChild);
        }

        /* NO modificar padding del body: el header es fixed y no consume espacio */
    }

    /* Primer intento inmediato */
    inject();
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', inject);
    }

    /* Watchdog cada 250ms: garantiza persistencia sin bucles infinitos */
    setInterval(inject, 250);
})();
            "#;

            let app_handle_nav = app.handle().clone();
            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://gemini.google.com".parse().unwrap()),
            )
            .title("Gemini Float")
            .inner_size(900.0, 700.0)
            .center()
            .resizable(true)
            .shadow(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(!is_dev)
            .visible(is_dev)
            .initialization_script(header_script)
            .on_navigation(move |url| {
                // Interceptar esquema gemini-float:// para acciones nativas de ventana.
                // La función devuelve `false` para cancelar la navegación antes de que
                // el webview cambie de página, manteniendo Gemini intacto.
                if url.scheme() == "gemini-float" {
                    if let Some(window) = app_handle_nav.get_webview_window("main") {
                        let path = url.path();
                        if path.contains("start-drag") {
                            // Inicia el arrastre nativo del sistema operativo
                            let _ = window.start_dragging();
                        } else if path.contains("close") {
                            let _ = window.close();
                        } else if path.contains("minimize") {
                            let _ = window.minimize();
                        } else if path.contains("toggle-maximize") {
                            let is_max = window.is_maximized().unwrap_or(false);
                            if is_max {
                                let _ = window.unmaximize();
                            } else {
                                let _ = window.maximize();
                            }
                        }
                    }
                    false // Cancelar navegación
                } else {
                    true // Permitir navegación normal en Gemini
                }
            })
            .build()?;

            // ── Atajo global Ctrl+Alt+Space ───────────────────────────
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
                eprintln!(
                    "Advertencia: No se pudo registrar Ctrl+Alt+Space: {}",
                    err
                );
            }

            // ── System Tray ───────────────────────────────────────────
            let show_item = MenuItemBuilder::with_id("show", "Mostrar/Ocultar").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Salir").build(app)?;

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
                    "quit" => app.exit(0),
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
