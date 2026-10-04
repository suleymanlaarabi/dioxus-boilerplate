//! Internal file API. Domain modules authorize access and reference files with real foreign keys.
use std::time::Duration;

use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use thiserror::Error;

pub const MAX_FILE_SIZE: usize = 25 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(pub i64);

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub id: FileId,
    pub original_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

pub struct NewFile {
    pub original_name: String,
    pub content_type: mime::Mime,
}

pub struct FileDownload {
    pub metadata: FileMetadata,
    pub content: ByteStream,
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("File name must contain 1 to 255 characters.")]
    InvalidName,
    #[error("File exceeds the 25 MiB limit.")]
    TooLarge,
    #[error("File not found.")]
    NotFound,
    #[error("File is still referenced by application data.")]
    Referenced,
    #[error("File storage failed: {0}")]
    Storage(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

struct FileRecord {
    id: i64,
    object_key: String,
    original_name: String,
    content_type: String,
    size_bytes: i64,
    created_at: DateTime<Utc>,
}

impl FileRecord {
    fn metadata(self) -> FileMetadata {
        FileMetadata {
            id: FileId(self.id),
            original_name: self.original_name,
            content_type: self.content_type,
            size_bytes: self.size_bytes,
            created_at: self.created_at,
        }
    }
}

#[derive(Clone)]
pub struct FileStore {
    db: PgPool,
    client: Client,
    bucket: String,
}

impl FileStore {
    pub async fn new(db: PgPool, config: &super::Config) -> Result<Self, FileError> {
        let client = Client::from_conf(
            aws_sdk_s3::config::Builder::new()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new(config.storage_region.clone()))
                .endpoint_url(&config.storage_endpoint)
                .credentials_provider(Credentials::new(
                    &config.storage_access_key,
                    &config.storage_secret_key,
                    None,
                    None,
                    "rustfs",
                ))
                .force_path_style(true)
                .timeout_config(
                    aws_sdk_s3::config::timeout::TimeoutConfig::builder()
                        .operation_timeout(Duration::from_secs(60))
                        .build(),
                )
                .build(),
        );
        let store = Self {
            db,
            client,
            bucket: config.storage_bucket.clone(),
        };
        if let Err(error) = store
            .client
            .head_bucket()
            .bucket(&store.bucket)
            .send()
            .await
        {
            if error
                .raw_response()
                .is_some_and(|response| response.status().as_u16() == 404)
            {
                store
                    .client
                    .create_bucket()
                    .bucket(&store.bucket)
                    .send()
                    .await
                    .map_err(storage_error)?;
            } else {
                return Err(storage_error(error));
            }
        }
        Ok(store)
    }

    pub async fn create(&self, input: NewFile, content: Bytes) -> Result<FileMetadata, FileError> {
        let name = input.original_name.trim();
        if !(1..=255).contains(&name.chars().count()) {
            return Err(FileError::InvalidName);
        }
        if content.len() > MAX_FILE_SIZE {
            return Err(FileError::TooLarge);
        }
        let size = i64::try_from(content.len()).map_err(|_| FileError::TooLarge)?;
        let key = super::auth::random_token().map_err(storage_error)?;
        let content_type = input.content_type.to_string();
        let mut transaction = self.db.begin().await?;
        let file = sqlx::query_as!(FileRecord,
            "INSERT INTO files (object_key, original_name, content_type, size_bytes)
             VALUES ($1, $2, $3, $4) RETURNING id, object_key, original_name, content_type, size_bytes, created_at",
            key, name, content_type, size).fetch_one(&mut *transaction).await?;
        sqlx::query!("INSERT INTO file_cleanup (object_key, due_at) VALUES ($1, now() + INTERVAL '15 minutes')", key)
            .execute(&mut *transaction).await?;
        transaction.commit().await?;

        // A persisted cleanup intent survives network failures and process interruption.
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(content_type)
            .body(ByteStream::from(content))
            .send()
            .await
            .map_err(storage_error)?;
        let mut transaction = self.db.begin().await?;
        sqlx::query!(
            "SELECT object_key FROM file_cleanup WHERE object_key = $1 FOR UPDATE",
            key
        )
        .fetch_one(&mut *transaction)
        .await?;
        sqlx::query!("UPDATE files SET ready = true WHERE id = $1", file.id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query!("DELETE FROM file_cleanup WHERE object_key = $1", key)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(file.metadata())
    }

    async fn find(&self, id: FileId) -> Result<Option<FileRecord>, FileError> {
        Ok(sqlx::query_as!(FileRecord,
            "SELECT id, object_key, original_name, content_type, size_bytes, created_at FROM files WHERE id = $1 AND ready",
            id.0).fetch_optional(&self.db).await?)
    }

    pub async fn get(&self, id: FileId) -> Result<Option<FileMetadata>, FileError> {
        Ok(self.find(id).await?.map(FileRecord::metadata))
    }

    pub async fn read(&self, id: FileId) -> Result<FileDownload, FileError> {
        let file = self.find(id).await?.ok_or(FileError::NotFound)?;
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&file.object_key)
            .send()
            .await
            .map_err(storage_error)?;
        Ok(FileDownload {
            metadata: file.metadata(),
            content: object.body,
        })
    }

    pub async fn delete(&self, id: FileId) -> Result<(), FileError> {
        let mut transaction = self.db.begin().await?;
        let deleted = sqlx::query!(
            "DELETE FROM files WHERE id = $1 AND ready RETURNING object_key",
            id.0
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            if error
                .as_database_error()
                .is_some_and(|error| error.is_foreign_key_violation())
            {
                FileError::Referenced
            } else {
                FileError::Database(error)
            }
        })?;
        let Some(file) = deleted else {
            return Ok(());
        };
        sqlx::query!(
            "INSERT INTO file_cleanup (object_key, due_at) VALUES ($1, now())",
            file.object_key
        )
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        if let Err(error) = self.clean_object(&file.object_key).await {
            dioxus::logger::tracing::warn!(%error, "File deletion queued for retry");
        }
        Ok(())
    }

    async fn clean_object(&self, key: &str) -> Result<(), FileError> {
        let mut transaction = self.db.begin().await?;
        let intent = sqlx::query!(
            "SELECT object_key FROM file_cleanup WHERE object_key = $1 AND due_at <= now() FOR UPDATE SKIP LOCKED", key)
            .fetch_optional(&mut *transaction).await?;
        if intent.is_none() {
            return Ok(());
        }
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(storage_error)?;
        sqlx::query!("DELETE FROM files WHERE object_key = $1 AND NOT ready", key)
            .execute(&mut *transaction)
            .await?;
        sqlx::query!("DELETE FROM file_cleanup WHERE object_key = $1", key)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub(super) async fn cleanup(&self) -> Result<(), FileError> {
        let intents = sqlx::query!(
            "SELECT object_key FROM file_cleanup WHERE due_at <= now() ORDER BY due_at LIMIT 100"
        )
        .fetch_all(&self.db)
        .await?;
        for intent in intents {
            if let Err(error) = self.clean_object(&intent.object_key).await {
                dioxus::logger::tracing::warn!(%error, "File cleanup will retry");
            }
        }
        Ok(())
    }
}

fn storage_error(error: impl std::fmt::Display) -> FileError {
    FileError::Storage(error.to_string())
}
