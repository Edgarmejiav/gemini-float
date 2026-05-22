#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Forzar el backend GDK a X11/XWayland en Linux para garantizar que la ventana flotante
    // sin bordes respete estrictamente la propiedad 'always_on_top' y nunca se vaya detrás de otras apps.
    #[cfg(target_os = "linux")]
    std::env::set_var("GDK_BACKEND", "x11");

    gemini_float_lib::run()
}
