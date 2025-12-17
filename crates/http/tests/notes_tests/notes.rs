use crate::common::{
    MockDbBuilder, assert_status_json, create_default_test_server, create_test_server_with_db,
    valid_access_token,
};
use crate::fake_data;
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_get_notes_requires_auth() {
    let server = create_default_test_server();
    let response = server.get("/api/v1/notes").await;

    // Missing Authorization header is treated as invalid token in extractor
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_notes_returns_notes_for_user() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let n1 = fake_data::note::model(Uuid::new_v4(), user_id, "first");
    let n2 = fake_data::note::model(Uuid::new_v4(), user_id, "second");

    let db = MockDbBuilder::new()
        .with_query_results::<entities::note::Model>(vec![vec![n1.clone(), n2.clone()]])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get("/api/v1/notes")
        .authorization_bearer(token)
        .await;

    let body = assert_status_json(response, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    let ids: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&n1.id.to_string()));
    assert!(ids.contains(&n2.id.to_string()));
}

#[tokio::test]
async fn test_get_note_by_id_returns_404_when_not_found() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);
    let note_id = Uuid::new_v4();

    let db = MockDbBuilder::new()
        .with_query_results::<entities::note::Model>(vec![vec![]])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get(&format!("/api/v1/notes/{note_id}"))
        .authorization_bearer(token)
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_note_returns_400_when_title_empty() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let server = create_default_test_server();
    let response = server
        .post("/api/v1/notes")
        .authorization_bearer(token)
        .json(&json!({
            "title": "   ",
            "archived": false,
            "pinned": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_note_returns_400_when_title_too_long() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let title = "a".repeat(501);
    let server = create_default_test_server();
    let response = server
        .post("/api/v1/notes")
        .authorization_bearer(token)
        .json(&json!({
            "title": title,
            "archived": false,
            "pinned": false
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_note_success_returns_201() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let inserted = fake_data::note::model(Uuid::new_v4(), user_id, "created");

    // ActiveModel::insert will require a mock query result (RETURNING ...)
    let db = MockDbBuilder::new()
        .with_query_results::<entities::note::Model>(vec![vec![inserted.clone()]])
        .with_exec_results(vec![sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .build();

    let server = create_test_server_with_db(db);
    let response = server
        .post("/api/v1/notes")
        .authorization_bearer(token)
        .json(&json!({
            "title": inserted.title,
            "archived": inserted.archived,
            "pinned": inserted.pinned
        }))
        .await;

    let body = assert_status_json(response, StatusCode::CREATED);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], inserted.id.to_string());
    assert_eq!(body["data"]["user_id"], inserted.user_id.to_string());
}
