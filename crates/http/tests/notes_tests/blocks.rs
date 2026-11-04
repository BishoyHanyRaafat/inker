use crate::common::{
    MockDbBuilder, assert_status_json, create_default_test_server, create_test_server_with_db,
    valid_access_token,
};
use crate::fake_data;
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_get_contents_by_note_id_requires_auth() {
    let server = create_default_test_server();
    let note_id = Uuid::new_v4();
    let response = server
        .get(&format!("/api/v1/notes/{note_id}/content"))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_contents_by_note_id_returns_blocks() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);
    let note_id = Uuid::new_v4();

    let b1 = fake_data::note_content::model_text(Uuid::new_v4(), note_id, 1, "hello".to_string());

    let db = MockDbBuilder::new()
        .with_query_results::<entities::note_content::Model>(vec![vec![b1.clone()]])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get(&format!("/api/v1/notes/{note_id}/content"))
        .authorization_bearer(token)
        .await;

    let body = assert_status_json(response, StatusCode::OK);

    assert_eq!(body["data"].get("content").unwrap(), "hello");
}

#[tokio::test]
async fn test_get_content_by_id_returns_404_when_not_found() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);
    let content_id = Uuid::new_v4();

    let db = MockDbBuilder::new()
        .with_tuple_query_results::<entities::note_content::Model, entities::note::Model>(vec![
            vec![],
        ])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get(&format!("/api/v1/notes/content/{content_id}"))
        .authorization_bearer(token)
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_content_by_id_returns_200_for_owned_note() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let note = fake_data::note::model(Uuid::new_v4(), user_id, "n");
    let content =
        fake_data::note_content::model_text(Uuid::new_v4(), note.id, 1, "hello".to_string());

    let db = MockDbBuilder::new()
        .with_tuple_query_results::<entities::note_content::Model, entities::note::Model>(vec![
            vec![(content.clone(), Some(note.clone()))],
        ])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get(&format!("/api/v1/notes/content/{}", note.id))
        .authorization_bearer(token)
        .await;

    let body = assert_status_json(response, StatusCode::OK);
    assert_eq!(body["data"]["note_id"], note.id.to_string());
}

#[tokio::test]
async fn test_get_content_by_id_returns_404_when_note_not_owned() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let other_user_id = Uuid::new_v4();
    let note = fake_data::note::model(Uuid::new_v4(), other_user_id, "n");
    let content =
        fake_data::note_content::model_text(Uuid::new_v4(), note.id, 1, "hello".to_string());

    let db = MockDbBuilder::new()
        .with_tuple_query_results::<entities::note_content::Model, entities::note::Model>(vec![
            vec![(content.clone(), Some(note.clone()))],
        ])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .get(&format!("/api/v1/notes/content/{}", note.id))
        .authorization_bearer(token)
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_content_returns_404_when_note_not_found() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);
    let note_id = Uuid::new_v4();

    let db = MockDbBuilder::new()
        .with_query_results::<entities::note::Model>(vec![vec![]]) // note lookup
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .post("/api/v1/notes/content")
        .authorization_bearer(token)
        .json(&json!({
            "note_id": note_id,
            "version": 1,
            "content": "new markdown"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_content_success_returns_201() {
    let user_id = Uuid::new_v4();
    let token = valid_access_token(user_id);

    let note = fake_data::note::model(Uuid::new_v4(), user_id, "n");
    let inserted_content =
        fake_data::note_content::model_text(Uuid::new_v4(), note.id, 1, "hello".to_string());

    let db = MockDbBuilder::new()
        // note lookup (ownership check)
        .with_query_results::<entities::note::Model>(vec![vec![note.clone()]])
        // inserted content (RETURNING ...)
        .with_query_results::<entities::note_content::Model>(vec![vec![inserted_content.clone()]])
        .with_exec_results(vec![sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .build();
    let server = create_test_server_with_db(db);

    let response = server
        .post("/api/v1/notes/content")
        .authorization_bearer(token)
        .json(&json!({
            "note_id": note.id,
            "version": inserted_content.version,
            "content": "content"
        }))
        .await;

    let body = assert_status_json(response, StatusCode::CREATED);
    assert_eq!(body["data"]["note_id"], note.id.to_string());
}
