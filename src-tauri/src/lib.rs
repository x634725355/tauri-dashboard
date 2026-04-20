mod dashboard;
mod platform;
mod snapshot_types;

use dashboard::{get_dashboard_snapshot, set_brightness_cmd, set_system_volume_cmd, NetCounters};
use parking_lot::Mutex;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .manage(Mutex::new(NetCounters::default()))
        .setup(|app| {
            use tauri_plugin_autostart::ManagerExt;
            if let Err(e) = app.autolaunch().enable() {
                eprintln!("[autostart] enable failed: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard_snapshot,
            set_system_volume_cmd,
            set_brightness_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
