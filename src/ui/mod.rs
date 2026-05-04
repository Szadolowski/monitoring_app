use crate::state::SharedState;
use crate::config::{self, AppConfig};
use eframe::egui;
use std::process::Command;

pub mod toast;

pub struct MonitorApp {
    state: SharedState,
    config: AppConfig,
    
    rt_handle: tokio::runtime::Handle,
    db_url: String,
    
    // Zmienna, która dzierży władzę życia i śmierci procesów w tle
    worker_handle: Option<tokio::task::JoinHandle<()>>,
    
    // Zaktualizowana krotka: (ID, Nazwa, PIN Hash, Opis)
    selected_profile: Option<(i32, String, String, String)>,
    password_input: String,
    auth_error: String,
}

impl MonitorApp {
    pub fn new(
        state: SharedState, 
        config: AppConfig, 
        rt_handle: tokio::runtime::Handle, 
        db_url: String,
        worker_handle: Option<tokio::task::JoinHandle<()>>
    ) -> Self {
        Self { 
            state, 
            config, 
            rt_handle, 
            db_url,
            worker_handle,
            selected_profile: None,
            password_input: String::new(),
            auth_error: String::new(),
        }
    }
}

impl eframe::App for MonitorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        // 1. EKRAN: Onboarding (Wybór Profilu i Logowanie)
        if state.current_profile_id == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading(egui::RichText::new("Witaj w systemie File Monitor!").size(24.0));
                ui.add_space(20.0);

                // EKRAN WPISYWANIA HASŁA
                if let Some((id, name, correct_pin_hash, desc)) = &self.selected_profile {
                    ui.label(egui::RichText::new(format!("Logowanie do: {}", name)).strong().size(18.0));
                    
                    // WYŚWIETLANIE OPISU Z BAZY DANYCH
                    if !desc.is_empty() {
                        ui.label(egui::RichText::new(desc).italics().color(egui::Color32::GRAY));
                    }
                    
                    ui.add_space(10.0);
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true).hint_text("Wprowadź kod PIN..."));
                    ui.add_space(10.0);

                    if !self.auth_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, &self.auth_error);
                        ui.add_space(10.0);
                    }

                    if ui.button(egui::RichText::new("Odblokuj Profil").size(16.0)).clicked() {
                        let is_valid = bcrypt::verify(&self.password_input, correct_pin_hash).unwrap_or(false);

                        if is_valid {
                            tracing::info!("Poprawne hasło! Inicjalizacja Gorącego Przeładowania...");
                            
                            self.config.profile_id = *id;
                            self.config.profile_name = name.clone();
                            let _ = config::save_local_config(&self.config);
                            state.current_profile_id = *id;

                            let sync_state = self.state.clone();
                            let watcher_state = self.state.clone();
                            let mut bg_config = self.config.clone();
                            let url = self.db_url.clone();
                            let profile_id = *id;

                            let handle = self.rt_handle.spawn(async move {
                                let client_opt = crate::db::DbClient::new(&url).await.ok();

                                if let Some(client) = &client_opt {
                                    if let Ok(paths) = client.get_paths_for_profile(profile_id).await {
                                        bg_config.watched_paths = paths.clone();
                                        let _ = crate::config::save_local_config(&bg_config);
                                        sync_state.lock().unwrap().watched_folders = paths.into_iter().map(|p| p.path).collect();
                                    }
                                }

                                let sync_future = async {
                                    if let Some(client) = client_opt {
                                        crate::sync::start_sync_worker(client, bg_config.clone(), sync_state).await;
                                    }
                                };

                                let watcher_future = async {
                                    let _ = crate::watcher::start_watching(&bg_config, watcher_state).await;
                                };

                                tokio::join!(sync_future, watcher_future);
                            });

                            self.worker_handle = Some(handle);

                        } else {
                            self.auth_error = "Nieprawidłowe hasło! Spróbuj ponownie.".to_string();
                        }
                    }

                    ui.add_space(20.0);
                    if ui.button("Wróć do listy").clicked() {
                        self.selected_profile = None;
                        self.password_input.clear();
                        self.auth_error.clear();
                    }

                } else {
                    // EKRAN WYBORU PROFILU
                    ui.label("Wybierz swój profil:");
                    ui.add_space(10.0);
                    if state.available_profiles.is_empty() {
                        ui.colored_label(egui::Color32::RED, "Błąd: Brak połączenia z bazą.");
                    } else {
                        let profiles = state.available_profiles.clone();
                        for profile in profiles {
                            if ui.button(egui::RichText::new(format!("👤 {}", profile.1)).size(18.0)).clicked() {
                                self.selected_profile = Some(profile.clone());
                            }
                            
                            // WYŚWIETLANIE OPISU POD PRZYCISKIEM
                            if !profile.3.is_empty() {
                                ui.label(egui::RichText::new(&profile.3).italics().small().color(egui::Color32::DARK_GRAY));
                            }
                            ui.add_space(5.0);
                        }
                    }
                }
            });
            return;
        }

        // 2. EKRAN: Główny Dashboard
        ui.horizontal(|ui| {
            ui.heading(format!("Panel Kontrolny - {}", self.config.profile_name));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Wyloguj / Zmień profil").clicked() {
                    
                    if let Some(handle) = self.worker_handle.take() {
                        handle.abort();
                        tracing::info!("Procesy tła (Watcher i Sync) zostały pomyślnie zabite.");
                    }

                    state.current_profile_id = 0;
                    state.events.clear();
                    state.watched_folders.clear();
                    
                    self.selected_profile = None;
                    self.password_input.clear();
                    self.auth_error.clear();

                    self.config.profile_id = 0;
                    self.config.profile_name = String::new();
                    self.config.watched_paths.clear();
                    let _ = config::save_local_config(&self.config);
                    
                    let url = self.db_url.clone();
                    let state_clone = self.state.clone();
                    self.rt_handle.spawn(async move {
                        if let Ok(client) = crate::db::DbClient::new(&url).await {
                            if let Ok(profiles) = client.get_profiles().await {
                                state_clone.lock().unwrap().available_profiles = profiles;
                            }
                        }
                    });
                }
            });
        });
        
        ui.separator();

        ui.label(egui::RichText::new("Śledzone foldery:").strong().size(16.0));
        if state.watched_folders.is_empty() {
            ui.label(egui::RichText::new("Pobieranie najnowszych katalogów z serwera...").italics());
        } else {
            for folder in &state.watched_folders {
                ui.label(format!("📁 {}", folder));
            }
        }

        ui.add_space(20.0);
        ui.separator();

        ui.label(egui::RichText::new("Ostatnie zdarzenia:").strong().size(16.0));
        
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            if state.events.is_empty() {
                ui.label("Brak nowych zdarzeń w bieżącej sesji.");
            }
            for event in state.events.iter() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("[{}]", event.timestamp)).monospace());
                    ui.label(egui::RichText::new(&event.event_type).color(egui::Color32::LIGHT_BLUE));
                    
                    if ui.button("Otwórz").clicked() {
                        if let Err(e) = Command::new("explorer").args(["/select,", &event.file_path]).spawn() {
                            tracing::error!("Błąd otwierania eksploratora: {}", e);
                        }
                    }
                    ui.label(&event.file_path);
                });
            }
        });

        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    }
}