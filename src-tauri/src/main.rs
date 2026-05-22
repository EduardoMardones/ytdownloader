#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[tauri::command]
async fn download_video(app: tauri::AppHandle, url: String, format: String, path: String) -> Result<String, String> {
    let output_template = format!("{}/%(title)s.%(ext)s", path);
    let mut args: Vec<String> = vec![url, "-o".to_string(), output_template];

    if format == "mp3" {
        args.extend(vec!["-x".to_string(), "--audio-format".to_string(), "mp3".to_string()]);
    } else if format == "mp4" {
        args.extend(vec!["-f".to_string(), "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]".to_string()]);
    }

    let (mut rx, _child) = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| e.to_string())?
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            println!("{}", String::from_utf8_lossy(&line));
        }
    }
    Ok("¡Descarga Finalizada!".into())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_video]) // Quitamos select_folder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
