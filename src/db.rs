use crate::config::WatchedPath;
use crate::error::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    /// Inicjalizuje pulę połączeń z rygorystycznymi limitami czasowymi
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(3) // To lekki agent, nie potrzebuje wielu połączeń
            .acquire_timeout(Duration::from_secs(5)) // Szybki upadek (fail-fast) przy braku sieci
            .connect(database_url)
            .await?;

        tracing::info!("Pomyślnie połączono z bazą PostgreSQL.");
        Ok(Self { pool })
    }

    /// Pobiera listę profili (używane podczas pierwszego uruchomienia/Onboardingu)
    pub async fn get_profiles(&self) -> Result<Vec<(i32, String, String, String)>> {
        let records = sqlx::query(
            "SELECT id, profile_name, pin_code, COALESCE(description, '') as description FROM public.monitoring_profiles"
        )
        .fetch_all(&self.pool)
        .await?;

        let profiles = records
            .into_iter()
            .map(|row| {
                let id: i32 = row.get("id");
                let name: String = row.get("profile_name");
                let pin_hash: String = row.try_get("pin_code").unwrap_or_default();
                let desc: String = row.get("description"); // Nasz nowy opis
                (id, name, pin_hash, desc)
            })
            .collect();

        Ok(profiles)
    }

    /// Pobiera aktywne ścieżki do monitorowania dla wybranego profilu
    pub async fn get_paths_for_profile(&self, profile_id: i32) -> Result<Vec<WatchedPath>> {
        let records = sqlx::query(
            "SELECT id, path, notification_msg FROM public.watched_paths WHERE profile_id = $1 AND is_active = true"
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await?;

        let paths = records
            .into_iter()
            .map(|row| {
                // Rzutujemy duże liczby z Postgresa (bigint) na i64
                let id: i64 = row.get("id");
                let path: String = row.get("path");
                let notification_msg: Option<String> = row.try_get("notification_msg").ok();

                WatchedPath {
                    id,
                    path,
                    notification_msg,
                }
            })
            .collect();

        Ok(paths)
    }
}