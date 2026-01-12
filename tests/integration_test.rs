use axum_test::TestServer;
use tulpar_api::{create_router, db, AppState};
use std::sync::Arc;
use tempfile::tempdir;
use std::fs;
use lz4_flex::frame::FrameEncoder;
use std::io::Write;
use serde_json::json;

#[tokio::test]
async fn test_full_api_flow() {
    let _ = tracing_subscriber::fmt::try_init();
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();
    let collections_dir = storage_path.join("collections");
    fs::create_dir_all(&collections_dir).expect("Failed to create collections dir");

    let db_path_file = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path_file.to_str().unwrap());
    
    let pool = db::establish_connection(&database_url).await.expect("Failed to connect to DB");
    db::run_migrations(&pool).await.expect("Failed to run migrations");

    let alias = "test-collection";
    let filename = format!("{}.json.lz4", alias);
    let file_path = collections_dir.join(&filename);
    
    let test_content = json!({
        "id": "coll-1",
        "name": "Test Collection",
        "items": [
            {
                "id": "item-1",
                "title": "Test Item 1",
                "body": "This is a test item body"
            }
        ]
    });

    let output_file = fs::File::create(&file_path).expect("Failed to create test file");
    let mut encoder = FrameEncoder::new(output_file);
    encoder.write_all(serde_json::to_string(&test_content).unwrap().as_bytes()).expect("Failed to compress");
    encoder.finish().expect("Failed to finish compression");

    let db_path_for_api = format!("collections/{}", filename);
    sqlx::query("INSERT INTO collections (alias, name, file_path, language) VALUES (?, ?, ?, ?)")
        .bind(alias)
        .bind("Test Collection")
        .bind(db_path_for_api)
        .bind("en")
        .execute(&pool)
        .await
        .expect("Failed to insert seed data");

    let state = Arc::new(AppState::new(pool, storage_path));
    let app = create_router(state);
    let server = TestServer::new(app).expect("Failed to create test server");

    let response = server.get("/api/v1/content/collections")
        .add_header(http::header::HeaderName::from_static("x-forwarded-for"), http::HeaderValue::from_static("127.0.0.1"))
        .await;
    let collections: serde_json::Value = response.json();
    let test_coll = collections.as_array().unwrap().iter().find(|c| c["alias"] == alias).expect("Test collection not found in list");
    assert_eq!(test_coll["alias"], alias);

    let response = server.get(&format!("/api/v1/content/collections/{}", alias))
        .add_header(http::header::HeaderName::from_static("x-forwarded-for"), http::HeaderValue::from_static("127.0.0.1"))
        .await;
    response.assert_status_ok();
    let collection: serde_json::Value = response.json();
    assert_eq!(collection["name"], "Test Collection");
    assert_eq!(collection["items"][0]["id"], "item-1");

    let response = server.get(&format!("/api/v1/content/collections/{}/items/item-1", alias))
        .add_header(http::header::HeaderName::from_static("x-forwarded-for"), http::HeaderValue::from_static("127.0.0.1"))
        .await;
    response.assert_status_ok();
    let item: serde_json::Value = response.json();
    assert_eq!(item["id"], "item-1");
    assert_eq!(item["title"], "Test Item 1");

    let response = server.get("/api/v1/content/collections/non-existent")
        .add_header(http::header::HeaderName::from_static("x-forwarded-for"), http::HeaderValue::from_static("127.0.0.1"))
        .await;
    response.assert_status_not_found();

    let response = server.get("/api/v1/content/collections")
        .add_header(http::header::HeaderName::from_static("x-forwarded-for"), http::HeaderValue::from_static("127.0.0.1"))
        .await;
    assert!(response.headers().contains_key("x-request-id"));
}
