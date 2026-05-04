use crate::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::env;

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

/// Pobiera adres bazy danych z .env lub używa domyślnego (fallback)
pub fn get_database_url() -> String {
    // 1. Spróbuj wczytać .env (zadziała w dev, w MSI zazwyczaj nie)
    dotenvy::dotenv().ok();

    // 2. Pobierz DATABASE_URL z systemu. 
    // JEŚLI NIE MA: użyj wpisanego na sztywno adresu IP Twojego serwera.
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@192.168.1.100/db_name".to_string())
}

/// Helper do pobierania bezpiecznej ścieżki w %APPDATA%
pub fn get_config_path() -> Result<PathBuf> {
    // Te wartości ukształtują ścieżkę: AppData/Roaming/MojaFirma/FileMonitorAgent
    let proj_dirs = ProjectDirs::from("com", "MojaFirma", "FileMonitorAgent")
        .ok_or_else(|| AppError::Config("Nie można zlokalizować katalogu AppData w systemie".to_string()))?;

    let config_dir = proj_dirs.config_dir();
    
    // Upewniamy się, że folder istnieje, zanim spróbujemy w nim cokolwiek zapisać
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).map_err(|e| AppError::Config(e.to_string()))?;
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

    let file = fs::File::open(path).map_err(|e| AppError::Config(e.to_string()))?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader).map_err(|e| AppError::Config(e.to_string()))?;

    tracing::info!("Konfiguracja załadowana pomyślnie z lokalnego dysku.");
    Ok(config)
}

/// Zapisuje konfigurację na dysk (używane po udanym pobraniu danych z bazy)
pub fn save_local_config(config: &AppConfig) -> Result<()> {
    let path = get_config_path()?;
    let file = fs::File::create(&path).map_err(|e| AppError::Config(e.to_string()))?;
    let writer = BufWriter::new(file);
    
    serde_json::to_writer_pretty(writer, config).map_err(|e| AppError::Config(e.to_string()))?;
    
    tracing::info!("Konfiguracja zapisana pomyślnie pod ścieżką: {:?}", path);
    Ok(())
}