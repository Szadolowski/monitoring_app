use crate::state::SharedState;
use crate::config::{self, AppConfig};
use eframe::egui;
use std::process::Command;
use std::collections::HashSet;

pub mod toast;

#[derive(Clone, PartialEq)]
pub enum OnboardingStep {
    SelectGroup,
    BiuroPassword,
    BiuroSelectProfiles,
    BrygadzistaSelectProfile,
    BrygadzistaPassword(i32, String, String, String), // id, name, hash, desc
    Dashboard,
}

pub struct MonitorApp {
    state: SharedState,
    config: AppConfig,
    
    rt_handle: tokio::runtime::Handle,
    db_url: String,
    worker_handle: Option<tokio::task::JoinHandle<()>>,
    
    // Zmienne UI dla nowego flow
    step: OnboardingStep,
    password_input: String,
    auth_error: String,
    
    // Listy pobrane z bazy
    biuro_profiles: Vec<(i32, String, String, String)>,
    brygadzista_profiles: Vec<(i32, String, String, String)>,
    
    // Zaznaczone checkboxy (tylko dla Biura)
    selected_biuro_ids: HashSet<i32>,
}

impl MonitorApp {
    pub fn new(
        state: SharedState, 
        config: AppConfig, 
        rt_handle: tokio::runtime::Handle, 
        db_url: String,
        worker_handle: Option<tokio::task::JoinHandle<()>>
    ) -> Self {
        
        let initial_step = if config.active_profile_ids.is_empty() {
            OnboardingStep::SelectGroup
        } else {
            OnboardingStep::Dashboard
        };

        let app = Self { 
            state, 
            config, 
            rt_handle, 
            db_url,
            worker_handle,
            step: initial_step,
            password_input: String::new(),
            auth_error: String::new(),
            biuro_profiles: Vec::new(),
            brygadzista_profiles: Vec::new(),
            selected_biuro_ids: HashSet::new(),
        };

        app.fetch_all_profiles();
        app
    }

    fn fetch_all_profiles(&self) {
        let url = self.db_url.clone();
        let state_clone = self.state.clone();
        
        self.rt_handle.spawn(async move {
            if let Ok(client) = crate::db::DbClient::new(&url).await {
                let biuro = client.get_profiles_by_group(1).await.unwrap_or_default();
                let brygadzista = client.get_profiles_by_group(2).await.unwrap_or_default();
                
                let mut st = state_clone.lock().unwrap();
                st.available_profiles = biuro; 
                st.available_brygadzista_profiles = brygadzista;
            }
        });
    }

    fn start_monitoring(&mut self, profile_ids: Vec<i32>, display_name: String) {
        tracing::info!("Uruchamianie monitorowania dla profili: {:?}", profile_ids);
        
        // NOWE: Zanim uruchomimy nowy proces, zabijamy stary 
        // (niezbędne do poprawnego działania funkcji "Odśwież")
        if let Some(handle) = self.worker_handle.take() {
            handle.abort();
        }

        self.config.active_profile_ids = profile_ids.clone();
        self.config.profile_name = display_name;
        let _ = config::save_local_config(&self.config);
        
        let sync_state = self.state.clone();
        let watcher_state = self.state.clone();
        let mut bg_config = self.config.clone();
        let url = self.db_url.clone();

        let handle = self.rt_handle.spawn(async move {
            let client_opt = crate::db::DbClient::new(&url).await.ok();

            if let Some(client) = &client_opt {
                if let Ok(paths) = client.get_paths_for_profiles(&profile_ids).await {
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
        self.step = OnboardingStep::Dashboard;
    }
}

impl eframe::App for MonitorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let state_arc = self.state.clone();
        let mut state = state_arc.lock().unwrap();

        if self.biuro_profiles.is_empty() && !state.available_profiles.is_empty() {
            self.biuro_profiles = state.available_profiles.clone();
        }
        if self.brygadzista_profiles.is_empty() && !state.available_brygadzista_profiles.is_empty() {
            self.brygadzista_profiles = state.available_brygadzista_profiles.clone();
        }

        ui.vertical_centered(|ui| {
            match &self.step {
                OnboardingStep::SelectGroup => {
                    ui.add_space(50.0);
                    ui.heading("Kim jesteś?");
                    ui.add_space(30.0);

                    if ui.button(egui::RichText::new("🏢 BIURO").size(24.0)).clicked() {
                        self.auth_error.clear();
                        self.password_input.clear();
                        self.step = OnboardingStep::BiuroPassword;
                    }
                    ui.add_space(20.0);
                    if ui.button(egui::RichText::new("👷 BRYGADZISTA").size(24.0)).clicked() {
                        self.step = OnboardingStep::BrygadzistaSelectProfile;
                    }
                }

                OnboardingStep::BiuroPassword => {
                    ui.add_space(50.0);
                    ui.heading("Logowanie do panelu Biuro");
                    ui.add_space(20.0);
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true).hint_text("Hasło główne biura..."));
                    
                    if !self.auth_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, &self.auth_error);
                    }

                    ui.add_space(10.0);
                    // Sprawdzamy czy naciśnięto Enter
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button(egui::RichText::new("Odblokuj").size(16.0)).clicked() || enter_pressed {
                        if let Some((_, _, hash, _)) = self.biuro_profiles.first() {
                            if bcrypt::verify(&self.password_input, hash).unwrap_or(false) {
                                self.step = OnboardingStep::BiuroSelectProfiles;
                            } else {
                                self.auth_error = "Nieprawidłowe hasło!".to_string();
                            }
                        } else {
                            self.auth_error = "Brak skonfigurowanych profili biura w bazie.".to_string();
                        }
                    }
                    
                    ui.add_space(20.0);
                    if ui.button("Wróć").clicked() { self.step = OnboardingStep::SelectGroup; }
                }

                OnboardingStep::BiuroSelectProfiles => {
                    ui.add_space(20.0);
                    ui.heading("Wybierz pakiety do monitorowania (możesz kilka):");
                    ui.add_space(10.0);

                    for (id, name, _, desc) in &self.biuro_profiles {
                        let mut is_selected = self.selected_biuro_ids.contains(id);
                        if ui.checkbox(&mut is_selected, name).changed() {
                            if is_selected { self.selected_biuro_ids.insert(*id); }
                            else { self.selected_biuro_ids.remove(id); }
                        }
                        if !desc.is_empty() {
                            ui.label(egui::RichText::new(desc).italics().small().color(egui::Color32::DARK_GRAY));
                        }
                        ui.add_space(5.0);
                    }

                    ui.add_space(20.0);
                    if !self.selected_biuro_ids.is_empty() {
                        if ui.button(egui::RichText::new("🚀 Uruchom Monitorowanie").strong().size(18.0)).clicked() {
                            let ids: Vec<i32> = self.selected_biuro_ids.iter().cloned().collect();
                            self.start_monitoring(ids, "Pakiet Biuro (Multiselect)".to_string());
                        }
                    }
                    ui.add_space(20.0);
                    if ui.button("Wyloguj").clicked() { self.step = OnboardingStep::SelectGroup; }
                }

                OnboardingStep::BrygadzistaSelectProfile => {
                    ui.add_space(30.0);
                    ui.heading("Wybierz swoją budowę:");
                    ui.add_space(10.0);

                    for profile in &self.brygadzista_profiles {
                        if ui.button(egui::RichText::new(format!("🏗️ {}", profile.1)).size(18.0)).clicked() {
                            self.password_input.clear();
                            self.auth_error.clear();
                            self.step = OnboardingStep::BrygadzistaPassword(profile.0, profile.1.clone(), profile.2.clone(), profile.3.clone());
                        }
                        if !profile.3.is_empty() {
                            ui.label(egui::RichText::new(&profile.3).italics().small().color(egui::Color32::DARK_GRAY));
                        }
                        ui.add_space(10.0);
                    }
                    
                    ui.add_space(20.0);
                    if ui.button("Wróć").clicked() { self.step = OnboardingStep::SelectGroup; }
                }

                OnboardingStep::BrygadzistaPassword(id, name, hash, _) => {
                    ui.add_space(50.0);
                    ui.heading(format!("Logowanie: {}", name));
                    ui.add_space(20.0);
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true).hint_text("Hasło projektu..."));
                    
                    if !self.auth_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, &self.auth_error);
                    }

                    ui.add_space(10.0);
                    // Sprawdzamy czy naciśnięto Enter
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button(egui::RichText::new("Odblokuj Budowę").size(16.0)).clicked() || enter_pressed {
                        if bcrypt::verify(&self.password_input, hash).unwrap_or(false) {
                            self.start_monitoring(vec![*id], name.clone());
                        } else {
                            self.auth_error = "Nieprawidłowe hasło budowy!".to_string();
                        }
                    }

                    ui.add_space(20.0);
                    if ui.button("Wróć").clicked() { self.step = OnboardingStep::BrygadzistaSelectProfile; }
                }

                OnboardingStep::Dashboard => {
                    ui.horizontal(|ui| {
                        ui.heading(format!("Panel Kontrolny - {}", self.config.profile_name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            
                            if ui.button("Wyloguj").clicked() {
                                if let Some(handle) = self.worker_handle.take() {
                                    handle.abort();
                                }
                                state.watched_folders.clear();
                                state.events.clear();
                                self.selected_biuro_ids.clear();
                                self.config.active_profile_ids.clear();
                                self.config.profile_name.clear();
                                let _ = config::save_local_config(&self.config);
                                
                                self.biuro_profiles.clear();
                                self.brygadzista_profiles.clear();
                                state.available_profiles.clear();
                                state.available_brygadzista_profiles.clear();
                                
                                self.fetch_all_profiles(); 
                                self.step = OnboardingStep::SelectGroup;
                            }

                            // NOWE: Przycisk odświeżania obok przycisku Wyloguj
                            if ui.button("🔄 Odśwież foldery").clicked() {
                                let ids = self.config.active_profile_ids.clone();
                                let name = self.config.profile_name.clone();
                                // Uruchamia logikę pobierania na nowo i resetuje proces w tle!
                                self.start_monitoring(ids, name);
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
                    
                    // NOWE: Nagłówek Zdarzeń z przyciskiem "Wyczyść wszystko"
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Ostatnie zdarzenia:").strong().size(16.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !state.events.is_empty() {
                                if ui.button("🗑 Wyczyść wszystko").clicked() {
                                    state.events.clear();
                                }
                            }
                        });
                    });
                    
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        if state.events.is_empty() {
                            ui.label("Brak nowych zdarzeń w bieżącej sesji.");
                        } else {
                            let mut index_to_remove = None;
                            
                            // Przechodzimy po wektorze odczytując też indeks (idx)
                            for (idx, event) in state.events.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    
                                    // NOWE: Przycisk X do usuwania pojedynczych zdarzeń
                                    if ui.button("❌").clicked() {
                                        index_to_remove = Some(idx);
                                    }
                                    
                                    ui.label(egui::RichText::new(format!("[{}]", event.timestamp)).monospace());
                                    ui.label(egui::RichText::new(&event.event_type).color(egui::Color32::LIGHT_BLUE));
                                    if ui.button("Otwórz").clicked() {
                                        let _ = Command::new("explorer").args(["/select,", &event.file_path]).spawn();
                                    }
                                    ui.label(&event.file_path);
                                });
                            }
                            
                            // Jeśli kliknięto w "X", usuwamy konkretny indeks po zakończeniu rysowania pętli
                            if let Some(idx) = index_to_remove {
                                state.events.remove(idx);
                            }
                        }
                    });
                }
            }
        });

        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    }
}