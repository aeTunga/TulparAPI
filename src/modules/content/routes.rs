use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use super::store::{CollectionMetadata, ContentCollection, ContentItem};
use crate::error::AppError;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/collections", get(list_collections))
        .route("/collections/:alias", get(get_collection))
        .route("/collections/:alias/items/:item_id", get(get_item))
}

async fn list_collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CollectionMetadata>>, AppError> {
    let collections = state.content_store.list_collections().await?;
    Ok(Json(collections))
}

async fn get_collection(
    State(state): State<Arc<AppState>>,
    Path(alias): Path<String>,
) -> Result<Json<Arc<ContentCollection>>, AppError> {
    tracing::info!("Hit get_collection for alias: {}", alias);
    let collection = state.content_store.get_collection(&alias).await?;
    Ok(Json(collection))
}

async fn get_item(
    State(state): State<Arc<AppState>>,
    Path((alias, item_id)): Path<(String, String)>,
) -> Result<Json<ContentItem>, AppError> {
    let item = state.content_store.get_item(&alias, &item_id).await?;
    Ok(Json(item))
}
