use crate::models::ProjectFile;
use std::fs;
use tauri_plugin_dialog::DialogExt;

/// Save a project to a JSON file.
#[tauri::command]
pub async fn save_project(
    app: tauri::AppHandle,
    project: ProjectFile,
) -> Result<String, String> {
    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("Failed to serialize project: {}", e))?;

    // Open save dialog
    let file_path = app
        .dialog()
        .file()
        .set_title("Save Project")
        .add_filter("Stymail Project", &["stymail.json"])
        .set_file_name("campaign.stymail.json")
        .blocking_save_file();

    let path = match file_path {
        Some(p) => p.into_path().map_err(|e| format!("Invalid path: {}", e))?,
        None => return Err("Save cancelled.".to_string()),
    };

    fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

/// Load a project from a JSON file.
#[tauri::command]
pub async fn load_project(app: tauri::AppHandle) -> Result<ProjectFile, String> {
    // Open file dialog
    let file_path = app
        .dialog()
        .file()
        .set_title("Load Project")
        .add_filter("Stymail Project", &["stymail.json", "json"])
        .blocking_pick_file();

    let path = match file_path {
        Some(p) => p.into_path().map_err(|e| format!("Invalid path: {}", e))?,
        None => return Err("Load cancelled.".to_string()),
    };

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let project: ProjectFile =
        serde_json::from_str(&content).map_err(|e| format!("Invalid project file: {}", e))?;

    Ok(project)
}

