#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod error;  // musi być pierwszy, jeśli inne moduły z niego korzystają
mod config; // teraz Rust znajdzie Twój plik config.rs
mod db;
mod state;
mod sync;
mod ui;
mod watcher;

use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use crate::state::AppState;
use eframe::egui;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Uruchamianie Agenta...");

    let _ = dotenvy::dotenv();

    let config = config::load_local_config().unwrap_or_else(|e| {
        tracing::warn!("Problem z konfiguracją: {}", e);
        config::AppConfig::default()
    });

    let rt = Runtime::new()?;
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("postgres://postgres:postgres@localhost:5432/postgres"));

    let mut biuro_profiles = Vec::new();
    let mut brygadzista_profiles = Vec::new();

    // Sprawdzamy czy lista wybranych profili jest pusta zamiast starego profile_id == 0
    if config.active_profile_ids.is_empty() {
        if let Ok(client) = rt.block_on(db::DbClient::new(&db_url)) {
            // Pobieramy profile dla obu grup
            if let Ok(profiles) = rt.block_on(client.get_profiles_by_group(1)) {
                biuro_profiles = profiles;
            }
            if let Ok(profiles) = rt.block_on(client.get_profiles_by_group(2)) {
                brygadzista_profiles = profiles;
            }
        }
    }

    let initial_folders: Vec<String> = config.watched_paths.iter().map(|wp| wp.path.clone()).collect();
    let app_state = Arc::new(Mutex::new(AppState {
        events: Vec::new(),
        watched_folders: initial_folders,
        current_profile_id: 0, // Zostawione do kompatybilności starszych funkcji UI
        available_profiles: biuro_profiles,
        available_brygadzista_profiles: brygadzista_profiles,
    }));

    let mut initial_worker = None;

    if !config.active_profile_ids.is_empty() {
        let state_for_watcher = app_state.clone();
        let state_for_sync = app_state.clone();
        let config_for_background = config.clone();
        let url_for_bg = db_url.clone();

        initial_worker = Some(rt.spawn(async move {
            let client_opt = db::DbClient::new(&url_for_bg).await.ok();

            let sync_future = async {
                if let Some(client) = client_opt {
                    crate::sync::start_sync_worker(client, config_for_background.clone(), state_for_sync).await;
                }
            };

            let watcher_future = async {
                let _ = watcher::start_watching(&config_for_background, state_for_watcher).await;
            };

            tokio::join!(sync_future, watcher_future);
        }));
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("File Monitor Agent"),
        ..Default::default()
    };

    let rt_handle = rt.handle().clone();

    eframe::run_native(
        "File Monitor Agent",
        options,
        Box::new(move |_cc| Ok(Box::new(ui::MonitorApp::new(app_state, config, rt_handle, db_url, initial_worker)))),
    ).map_err(|e| anyhow::anyhow!("Błąd eframe: {}", e))?;

    Ok(())
}