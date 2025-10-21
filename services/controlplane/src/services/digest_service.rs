use proto::api::controlplane::{GetDigestByNameResponse, SetDigestToNameResponse};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tracing::{debug, error, info, instrument, warn};
use std::path::Path;
use tonic::{Response, Status};

pub struct DigestService {
    pool: SqlitePool,
}

impl DigestService {
    #[instrument(skip(db_path), fields(db_path = %db_path.display()))]
    pub async fn new(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing DigestService");
        let parent = db_path
            .parent()
            .ok_or("Database path has no parent directory")?;

        if !parent.exists() {
            error!(parent_dir = %parent.display(), "Parent directory does not exist");
            return Err(format!("Parent directory does not exist: {}", parent.display()).into());
        }

        debug!(parent_dir = %parent.display(), "Parent directory exists");

        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
        debug!(database_url = %database_url, "Connecting to database");

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        info!("Database connection established");

        debug!("Creating digests table if not exists");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS digests (
                name TEXT PRIMARY KEY,
                digest TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create digests table");
            e
        })?;

        info!("DigestService initialized successfully");
        Ok(Self { pool })
    }

    #[instrument(skip(self), fields(key = %key))]
    pub async fn get_digest_by_name(
        &self,
        key: &str,
    ) -> Result<Response<GetDigestByNameResponse>, Status> {
        debug!("Fetching digest from database");
        let result = sqlx::query_as::<_, (String,)>("SELECT digest FROM digests WHERE name = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!(error = %e, "Database query failed");
                Status::internal(format!("Database error: {}", e))
            })?;

        match result {
            Some((digest,)) => {
                info!(digest_length = digest.len(), "Digest found");
                Ok(Response::new(GetDigestByNameResponse { digest }))
            }
            None => {
                warn!("Digest not found");
                Err(Status::not_found(format!(
                    "Digest not found for name: {}",
                    key
                )))
            }
        }}

    #[instrument(skip(self, digest), fields(key = %key, digest_length = digest.len()))]
    pub async fn set_digest_by_name(
        &self,
        key: &str,
        digest: &str,
    ) -> Result<Response<SetDigestToNameResponse>, Status> {
        debug!("Upserting digest into database");
        sqlx::query(
            r#"
            INSERT INTO digests (name, digest, updated_at)
            VALUES (?, ?, strftime('%s', 'now'))
            ON CONFLICT(name) DO UPDATE SET
                digest = excluded.digest,
                updated_at = strftime('%s', 'now')
            "#,
        )
        .bind(key)
        .bind(digest)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database upsert failed");
            Status::internal(format!("Database error: {}", e))
        })?;

        info!("Digest set successfully");
        Ok(Response::new(SetDigestToNameResponse { success: true }))
    }
}
