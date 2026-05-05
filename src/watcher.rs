use crate::config::AppConfig;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::path::Path;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc;
use walkdir::WalkDir;
use chrono::Local;

pub async fn start_watching(app_config: &AppConfig, state: crate::state::SharedState) -> anyhow::Result<()> {
    if app_config.watched_paths.is_empty() {
        tracing::info!("Watcher: Brak przypisanych ścieżek.");
        return Ok(());
    }

    // FAZA 1: SYSTEM CATCH-UP (SKANOWANIE ZALEGŁOŚCI OFFLINE)
    // Opcja 1: Jeśli to pierwsze uruchomienie (0), ignorujemy historię i robimy "kalibrację".
    if app_config.last_scan_time > 0 {
        tracing::info!("Wykonywanie skanowania wyrównawczego...");
        for wp in &app_config.watched_paths {
            let base_path = Path::new(&wp.path);
            if !base_path.exists() { continue; }

            for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file() {
                    let file_name = entry.file_name().to_string_lossy();
                    // FILTR TEMPORARY FILES
                    if file_name.starts_with('~') || file_name.ends_with(".tmp") {
                        continue;
                    }

                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                                // Sprawdzamy czy plik jest nowszy niż czas naszego ostatniego udanego skanu
                                if duration.as_secs() > app_config.last_scan_time {
                                    tracing::info!("Catch-up: Wykryto zaległy plik -> {:?}", entry.path());
                                    
                                    let now = Local::now().format("%H:%M:%S").to_string();
                                    state.lock().unwrap().events.push(crate::state::FileEvent {
                                        timestamp: now,
                                        event_type: "Zaległy plik".to_string(),
                                        file_path: entry.path().to_string_lossy().to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        tracing::info!("Pierwsze uruchomienie profilu (last_scan_time = 0). Wykonuję kalibrację bazy. Ignoruję istniejące pliki, aby uniknąć spamu powiadomień.");
    }

    // FAZA 2: STANDARDOWY NASŁUCH W CZASIE RZECZYWISTYM
    let (tx, mut rx) = mpsc::channel(100);
    let mut watcher = RecommendedWatcher::new(
        move |res| { let _ = tx.blocking_send(res); },
        Config::default(),
    )?;

    for wp in &app_config.watched_paths {
        let path = Path::new(&wp.path);
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            tracing::info!("Watcher: Nasłuchuję -> {}", wp.path);
        }
    }

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                let (should_notify, title) = match event.kind {
                    EventKind::Create(_) => (true, "Nowy plik"),
                    EventKind::Modify(notify::event::ModifyKind::Name(_)) => (true, "Zmieniono nazwę"),
                    _ => (false, ""),
                };

                if should_notify {
                    if let Some(path) = event.paths.last() {
                        if path.exists() && path.is_file() {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            
                            // FILTR TEMPORARY FILES W CZASIE RZECZYWISTYM
                            if file_name.starts_with('~') || file_name.ends_with(".tmp") {
                                continue;
                            }

                            tracing::info!("Watcher: Zdarzenie '{}' dla -> {:?}", title, path);
                            let now = Local::now().format("%H:%M:%S").to_string();
                            
                            // ZAPIS ZDARZENIA DO PAMIĘCI GUI
                            state.lock().unwrap().events.push(crate::state::FileEvent {
                                timestamp: now,
                                event_type: title.to_string(),
                                file_path: path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Błąd systemu plików: {:?}", e),
        }
    }

    Ok(())
}