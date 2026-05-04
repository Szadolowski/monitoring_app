use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Błąd wejścia/wyjścia: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Błąd parsowania JSON: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Błąd bazy danych: {0}")]
    Db(#[from] sqlx::Error),
    
    #[error("Błąd konfiguracji: {0}")]
    Config(String),
    
    #[error("Nieoczekiwany błąd: {0}")]
    Other(#[from] anyhow::Error),
}

// Tworzymy własny alias na Result, aby nie pisać wszędzie Result<T, AppError>
pub type Result<T> = std::result::Result<T, AppError>;