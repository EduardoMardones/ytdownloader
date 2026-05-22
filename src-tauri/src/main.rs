#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri::Emitter;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::fs;

#[tauri::command]
async fn update_engine(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_dir.exists() { fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?; }
    let target_path = app_dir.join("yt-dlp-updated");
    let status = Command::new("curl").arg("-L")
        .arg("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp")
        .arg("-o").arg(&target_path).status().map_err(|e| e.to_string())?;
    if status.success() {
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        }
        Ok("Motor actualizado con éxito.".into())
    } else { Err("Fallo al descargar.".into()) }
}

#[tauri::command]
async fn download_video(app: tauri::AppHandle, url: String, format: String, path: String) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().unwrap();
    let updated_engine = app_dir.join("yt-dlp-updated");

    // Seleccionar motor (Actualizado o Interno)
    let engine_path = if updated_engine.exists() { updated_engine } 
    else { 
        app.path().resolve("binaries/yt-dlp-x86_64-unknown-linux-gnu", tauri::path::BaseDirectory::Resource).unwrap() 
    };

    // Rutas a FFMPEG y FFPROBE (Indispensables para unir video y audio)
    let ffmpeg_path = app.path().resolve("binaries/ffmpeg-x86_64-unknown-linux-gnu", tauri::path::BaseDirectory::Resource).unwrap();
    let ffprobe_path = app.path().resolve("binaries/ffprobe-x86_64-unknown-linux-gnu", tauri::path::BaseDirectory::Resource).unwrap();

    let output_template = format!("{}/%(title)s.%(ext)s", path);
    
    let mut args = vec![
        url,
        "-o".to_string(), output_template,
        "--newline".to_string(),
        "--force-overwrites".to_string(), // Esto obliga a procesar aunque el archivo exista
    ];

    if format == "mp3" {
        args.extend(vec!["-x".to_string(), "--audio-format".to_string(), "mp3".to_string()]);
    } else if format == "mp4" {
        // Mejoramos el comando de formato para asegurar compatibilidad
        args.extend(vec!["-f".to_string(), "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]".to_string(), "--merge-output-format".to_string(), "mp4".to_string()]);
    }

    let mut child = Command::new(engine_path)
        .args(args)
        // VARIABLES DE ENTORNO: La clave para que yt-dlp encuentre ffmpeg dentro de la AppImage
        .env("FFMPEG", &ffmpeg_path)
        .env("FFPROBE", &ffprobe_path)
        // LIMPIEZA: Para evitar el error de Python en AppImage
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env_remove("LD_LIBRARY_PATH")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Error: {}", e))?;

    let stdout = child.stdout.take().ok_or("Error Salida")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains("[download]") && l.contains("%") {
                let parts: Vec<&str> = l.split_whitespace().collect();
                for part in parts {
                    if part.contains("%") {
                        if let Ok(pct) = part.replace("%", "").parse::<f32>() {
                            let _ = app.emit("download-progress", pct);
                        }
                    }
                }
            }
            println!("{}", l);
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() { Ok("¡Completado con éxito!".into()) } else { Err("Error en la descarga o unión.".into()) }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_video, update_engine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}