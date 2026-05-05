use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub timestamp: String,
    pub event_type: String,
    pub file_path: String,
}

#[derive(Default)]
pub struct AppState {
    pub events: Vec<FileEvent>,
    pub watched_folders: Vec<String>,
    pub current_profile_id: i32,
    pub available_profiles: Vec<(i32, String, String, String)>,
    pub available_brygadzista_profiles: Vec<(i32, String, String, String)>, 
}

pub type SharedState = Arc<Mutex<AppState>>;