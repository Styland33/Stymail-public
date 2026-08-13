mod engine;
mod models;
mod project;
mod smtp;
mod spintax;

use engine::EngineState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(EngineState::new())
        .invoke_handler(tauri::generate_handler![
            // SMTP
            smtp::test_smtp,
            // Engine
            engine::start_campaign,
            engine::toggle_pause,
            engine::stop_campaign,
            engine::get_stats,
            engine::get_logs,
            // Project
            project::save_project,
            project::load_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}