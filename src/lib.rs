pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod modules;

use axum::Router;
use db::DbPool;
use modules::content::ContentStore;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: DbPool,
    pub content_store: ContentStore,
}

impl AppState {
    pub fn new(db: DbPool, storage_path: PathBuf) -> Self {
        Self {
            content_store: ContentStore::new(storage_path, db.clone()),
            db,
        }
    }
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let (request_id_layer, propagate_layer) = middleware::request_id();

    Router::new()
        .nest("/api/v1/content", modules::content::routes())
        .layer(middleware::rate_limit())
        .layer(middleware::cors())
        .layer(propagate_layer)
        .layer(middleware::trace())
        .layer(request_id_layer)
        .with_state(state)
}
