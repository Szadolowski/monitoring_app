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
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        tracing::info!("Pomyślnie połączono z bazą PostgreSQL.");
        Ok(Self { pool })
    }

    /// Pobiera listę profili dla danej grupy (1 = Biuro, 2 = Brygadzista)
    pub async fn get_profiles_by_group(&self, group_id: i32) -> Result<Vec<(i32, String, String, String)>> {
        let records = sqlx::query(
            "SELECT id, profile_name, password_hash, COALESCE(description, '') as description 
             FROM public.monitoring_profiles 
             WHERE group_id = $1"
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        let profiles = records
            .into_iter()
            .map(|row| {
                let id: i32 = row.get("id");
                let name: String = row.get("profile_name");
                let hash: String = row.try_get("password_hash").unwrap_or_default();
                let desc: String = row.get("description");
                (id, name, hash, desc)
            })
            .collect();

        Ok(profiles)
    }

    /// Pobiera aktywne ścieżki dla JEDNEGO LUB WIELU profili naraz
    pub async fn get_paths_for_profiles(&self, profile_ids: &[i32]) -> Result<Vec<WatchedPath>> {
        // Używamy ANY($1), aby Postgres dopasował id profilu do naszej listy
        let records = sqlx::query(
            "SELECT id, path, notification_msg 
             FROM public.watched_paths 
             WHERE profile_id = ANY($1) AND is_active = true"
        )
        .bind(profile_ids)
        .fetch_all(&self.pool)
        .await?;

        let paths = records
            .into_iter()
            .map(|row| {
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