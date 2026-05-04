#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod error;
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

    let mut profiles_for_ui = Vec::new();
    if config.profile_id == 0 {
        if let Ok(client) = rt.block_on(db::DbClient::new(&db_url)) {
            if let Ok(profiles) = rt.block_on(client.get_profiles()) {
                profiles_for_ui = profiles;
            }
        }
    }

    let initial_folders: Vec<String> = config.watched_paths.iter().map(|wp| wp.path.clone()).collect();
    let app_state = Arc::new(Mutex::new(AppState {
        events: Vec::new(),
        watched_folders: initial_folders,
        current_profile_id: config.profile_id,
        available_profiles: profiles_for_ui,
    }));

    // NOWE: Przechowujemy opcjonalny uchwyt zadania w tle
    let mut initial_worker = None;

    if config.profile_id != 0 {
        let state_for_watcher = app_state.clone();
        let state_for_sync = app_state.clone();
        let config_for_background = config.clone();
        let url_for_bg = db_url.clone();

        // Przypisujemy zgrupowane procesy do jednego uchwytu (JoinHandle)
        initial_worker = Some(rt.spawn(async move {
            let client_opt = db::DbClient::new(&url_for_bg).await.ok();

            // Używamy złączenia futures (tokio::join!), aby żyły jako jedno zadanie nadrzędne
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

    // Przekazujemy initial_worker do UI
    eframe::run_native(
        "File Monitor Agent",
        options,
        Box::new(move |_cc| Ok(Box::new(ui::MonitorApp::new(app_state, config, rt_handle, db_url, initial_worker)))),
    ).map_err(|e| anyhow::anyhow!("Błąd eframe: {}", e))?;

    Ok(())
}