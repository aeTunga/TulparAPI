use crate::db::DbPool;
use crate::error::AppError;
use lz4_flex::frame::FrameDecoder;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCollection {
    pub id: String,
    pub name: String,
    pub items: Vec<ContentItem>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CollectionMetadata {
    pub id: i64,
    pub alias: String,
    pub name: String,
    pub file_path: String,
    pub language: Option<String>,
}

#[derive(Clone)]
pub struct ContentStore {
    cache: Cache<String, Arc<ContentCollection>>,
    storage_path: PathBuf,
    db: DbPool,
}

impl ContentStore {
    pub fn new(storage_path: PathBuf, db: DbPool) -> Self {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self {
            cache,
            storage_path,
            db,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_collection(&self, alias: &str) -> Result<Arc<ContentCollection>, AppError> {
        if let Some(cached) = self.cache.get(alias).await {
            debug!(alias, "Cache hit");
            return Ok(cached);
        }

        debug!(alias, "Cache miss, loading from storage");

        let meta = self.get_metadata(alias).await?;
        let file_path = self.storage_path.join(&meta.file_path);

        if !file_path.exists() {
            return Err(AppError::NotFound(format!(
                "Storage file not found: {:?}",
                file_path
            )));
        }

        let data = tokio::fs::read(&file_path).await?;

        let collection = tokio::task::spawn_blocking(move || {
            let mut decoder = FrameDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| AppError::Lz4(e.to_string()))?;
            serde_json::from_slice::<ContentCollection>(&decompressed).map_err(AppError::from)
        })
        .await
        .map_err(|e| AppError::TaskJoin(e.to_string()))??;

        let arc_collection = Arc::new(collection);
        self.cache
            .insert(alias.to_string(), arc_collection.clone())
            .await;

        info!(alias, "Loaded collection from disk");
        Ok(arc_collection)
    }

    async fn get_metadata(&self, alias: &str) -> Result<CollectionMetadata, AppError> {
        let result: Option<CollectionMetadata> =
            sqlx::query_as("SELECT id, alias, name, file_path, language FROM collections WHERE alias = ?")
                .bind(alias)
                .fetch_optional(&self.db)
                .await?;

        result.ok_or_else(|| {
            AppError::NotFound(format!("Collection metadata not found for alias: {}", alias))
        })
    }

    pub async fn get_item(
        &self,
        collection_alias: &str,
        item_id: &str,
    ) -> Result<ContentItem, AppError> {
        let collection = self.get_collection(collection_alias).await?;

        collection
            .items
            .iter()
            .find(|item| item.id == item_id)
            .cloned()
            .ok_or_else(|| {
                AppError::NotFound(format!("Item {} not found in {}", item_id, collection_alias))
            })
    }

    pub async fn list_collections(&self) -> Result<Vec<CollectionMetadata>, AppError> {
        let results: Vec<CollectionMetadata> =
            sqlx::query_as("SELECT id, alias, name, file_path, language FROM collections")
                .fetch_all(&self.db)
                .await?;
        Ok(results)
    }
}
