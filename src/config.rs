use crate::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Reprezentuje pojedynczą ścieżkę do monitorowania (odwzorowanie z bazy)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchedPath {
    pub id: i64,
    pub path: String,
    pub notification_msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub profile_id: i32,
    pub profile_name: String,
    pub watched_paths: Vec<WatchedPath>,
    #[serde(default)]
    pub last_scan_time: u64, // Pamięta czas (Unix) ostatniego działania
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profile_id: 0,
            profile_name: "Nieprzypisany".to_string(),
            watched_paths: Vec::new(),
            last_scan_time: 0, 
        }
    }
}

/// Helper do pobierania bezpiecznej ścieżki w %APPDATA%
fn get_config_path() -> Result<PathBuf> {
    // Te wartości ukształtują ścieżkę: AppData/Roaming/MojaFirma/FileMonitorAgent
    let proj_dirs = ProjectDirs::from("com", "MojaFirma", "FileMonitorAgent")
        .ok_or_else(|| AppError::Config("Nie można zlokalizować katalogu AppData w systemie".to_string()))?;

    let config_dir = proj_dirs.config_dir();
    
    // Upewniamy się, że folder istnieje, zanim spróbujemy w nim cokolwiek zapisać
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    Ok(config_dir.join("config.json"))
}

/// Wczytuje konfigurację; jeśli plik nie istnieje, zwraca domyślną strukturę.
pub fn load_local_config() -> Result<AppConfig> {
    let path = get_config_path()?;

    if !path.exists() {
        tracing::info!("Brak lokalnego pliku konfiguracyjnego. Ładowanie ustawień domyślnych.");
        return Ok(AppConfig::default());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;

    tracing::info!("Konfiguracja załadowana pomyślnie z lokalnego dysku.");
    Ok(config)
}

/// Zapisuje konfigurację na dysk (używane po udanym pobraniu danych z bazy)
pub fn save_local_config(config: &AppConfig) -> Result<()> {
    let path = get_config_path()?;
    let file = fs::File::create(&path)?;
    let writer = BufWriter::new(file);
    
    serde_json::to_writer_pretty(writer, config)?;
    
    tracing::info!("Konfiguracja zapisana pomyślnie pod ścieżką: {:?}", path);
    Ok(())
}