#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri::Emitter;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::fs;

// FUNCIÓN PARA ACTUALIZAR EL MOTOR (yt-dlp)
#[tauri::command]
async fn update_engine(app: tauri::AppHandle) -> Result<String, String> {
    // 1. Obtener la carpeta de datos de la app (ej: ~/.local/share/com.eduardo.ytdownloader)
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    
    let target_path = app_dir.join("yt-dlp-updated");

    // 2. Descargar el binario directamente desde GitHub usando 'curl'
    // Usamos el sistema para no añadir dependencias pesadas a Rust
    let status = Command::new("curl")
        .arg("-L")
        .arg("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp")
        .arg("-o")
        .arg(&target_path)
        .status()
        .map_err(|e| format!("Error de conexión: {}", e))?;

    if status.success() {
        // 3. Dar permisos de ejecución (Necesario en Linux)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        }
        Ok("Motor actualizado con éxito. Ahora la app es compatible con los últimos cambios de YouTube.".into())
    } else {
        Err("No se pudo descargar la actualización. Verifica tu internet.".into())
    }
}

#[tauri::command]
async fn download_video(app: tauri::AppHandle, url: String, format: String, path: String) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().unwrap();
    let updated_engine = app_dir.join("yt-dlp-updated");

    // LÓGICA DE SELECCIÓN: Si existe la versión actualizada, úsala. 
    // Si no, usa la que viene de fábrica en la AppImage.
    let engine_path = if updated_engine.exists() {
        println!("Usando motor actualizado en: {:?}", updated_engine);
        updated_engine
    } else {
        app.path().resolve("binaries/yt-dlp-x86_64-unknown-linux-gnu", tauri::path::BaseDirectory::Resource)
            .map_err(|_| "No se encontró ningún motor de descarga disponible".to_string())?
    };

    let ffmpeg_res = app.path().resolve("binaries/ffmpeg-x86_64-unknown-linux-gnu", tauri::path::BaseDirectory::Resource);
    let output_template = format!("{}/%(title)s.%(ext)s", path);

    let mut args = vec![url.clone(), "-o".to_string(), output_template, "--newline".to_string()];

    if let Ok(ff_path) = ffmpeg_res {
        args.push("--ffmpeg-location".to_string());
        args.push(ff_path.to_string_lossy().to_string());
    }

    if format == "mp3" {
        args.extend(vec!["-x".to_string(), "--audio-format".to_string(), "mp3".to_string()]);
    } else if format == "mp4" {
        args.extend(vec!["-f".to_string(), "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]".to_string()]);
    }

    let mut child = Command::new(engine_path)
        .args(args)
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH")
        .env_remove("LD_LIBRARY_PATH")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Error al iniciar: {}", e))?;

    let stdout = child.stdout.take().ok_or("Error salida")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.contains("[download]") && l.contains("%") {
                let parts: Vec<&str> = l.split_whitespace().collect();
                for part in parts {
                    if part.contains("%") {
                        let pct_str = part.replace("%", "");
                        if let Ok(pct) = pct_str.parse::<f32>() {
                            let _ = app.emit("download-progress", pct);
                        }
                    }
                }
            }
            println!("{}", l);
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() { Ok("¡Descarga completada!".into()) } else { Err("Error en el proceso de descarga.".into()) }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_video, update_engine]) // Añadimos la nueva función
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}