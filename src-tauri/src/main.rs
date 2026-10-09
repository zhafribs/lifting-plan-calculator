// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's DMA-BUF renderer can kill the process on some NVIDIA driver
    // combinations before the window ever draws ("Error 71 (Protocol error)
    // dispatching to Wayland display"). When the NVIDIA kernel module is loaded
    // and the operator has not made their own choice, fall back to the
    // software path so the window always comes up. The cost is unmeasurable
    // for a form-based UI; the alternative is an AppImage that crashes on
    // exactly the machines its users have.
    #[cfg(target_os = "linux")]
    {
        let chosen = std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some();
        if !chosen && std::path::Path::new("/sys/module/nvidia").exists() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    lifting_plan_calculator_lib::run()
}
