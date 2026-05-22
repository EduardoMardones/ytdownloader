// Acceso a las funciones globales de Tauri v2
const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog; // Nuevo: Acceso directo al plugin de diálogo

document.addEventListener('DOMContentLoaded', () => {
    const btnDownload = document.getElementById('btn-download');
    const btnPath = document.getElementById('btn-select-path');
    const btnUpdate = document.getElementById('btn-update');
    const statusText = document.getElementById('status');
    const progressBar = document.getElementById('progress-bar');
    const currentPathDisplay = document.getElementById('current-path');
    
    let selectedPath = "";

    // Seleccionar carpeta
    btnPath.addEventListener('click', async () => {
        try {
            const path = await open({
                directory: true,
                multiple: false,
                title: "Selecciona la carpeta de destino"
            });

            if (path) {
                selectedPath = path;
                currentPathDisplay.innerText = path;
                statusText.innerText = "Carpeta seleccionada correctamente";
            }
        } catch (err) {
            console.error("Error al abrir el diálogo:", err);
        }
    });

    // Actualizar motor (Cambios aplicados aquí)
    btnUpdate.addEventListener('click', async () => {
        statusText.innerText = "⏳ Descargando última versión del motor... No cierres la app.";
        statusText.style.color = "orange";
        
        try {
            const res = await invoke('update_engine');
            statusText.innerText = "✅ " + res;
            statusText.style.color = "green";
        } catch (e) {
            statusText.innerText = "❌ Error al actualizar: " + e;
            statusText.style.color = "red";
        }
    });

    // Descargar video
    btnDownload.addEventListener('click', async () => {
        const url = document.getElementById('url').value;
        const format = document.getElementById('format').value;

        if (!url || !selectedPath) {
            statusText.innerText = "⚠️ Falta URL o Carpeta";
            statusText.style.color = "red";
            return;
        }

        statusText.innerText = "⏳ Descargando...";
        statusText.style.color = "black";
        progressBar.style.width = "30%";

        try {
            const res = await invoke('download_video', { url, format, path: selectedPath });
            statusText.innerText = "✅ " + res;
            statusText.style.color = "green";
            progressBar.style.width = "100%";
        } catch (e) {
            statusText.innerText = "❌ Error: " + e;
            statusText.style.color = "red";
            progressBar.style.width = "0%";
        }
    });
});