use crate::config::{self, AppConfig};
use crate::db::DbClient;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

pub async fn start_sync_worker(client: DbClient, mut config: AppConfig, state: crate::state::SharedState) {
    let mut interval = time::interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;

        if config.profile_id == 0 {
            tracing::info!("Brak profilu. Worker synchronizacji usypia.");
            continue; 
        }

        tracing::info!("Rozpoczynam synchronizację dla profilu: {}", config.profile_name);

        match client.get_paths_for_profile(config.profile_id).await {
            Ok(paths) => {
                tracing::info!("Pobrano {} ścieżek z bazy.", paths.len());
                
                // PRZEKAZANIE FOLDERÓW DO PAMIĘCI GUI
                state.lock().unwrap().watched_folders = paths.iter().map(|p| p.path.clone()).collect();
                config.watched_paths = paths;

                config.last_scan_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Err(e) = config::save_local_config(&config) {
                    tracing::error!("Błąd zapisu cache: {}", e);
                } else {
                    tracing::info!("Synchronizacja zakończona sukcesem.");
                }
            }
            Err(e) => {
                tracing::error!("Błąd sieci/bazy danych: {}", e);
            }
        }
    }
}