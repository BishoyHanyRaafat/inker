use reqwest::StatusCode;
use sea_orm::MockExecResult;
use serde_json::json;

use crate::common::{
    MockDbBuilder, assert_status_json, create_default_test_server, create_test_server_with_db,
};
use crate::fake_data;

/// Test that signup route rejects GET requests
#[tokio::test]
async fn test_signup_rejects_get_method() {
    let server = create_default_test_server();

    let response = server.get("/api/v1/auth/signup").await;

    // Should return 405 Method Not Allowed for GET on signup
    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
}

/// Test that signup returns 422 for completely empty JSON body
#[tokio::test]
async fn test_signup_empty_json_returns_unprocessable() {
    let server = create_default_test_server();

    let response = server.post("/api/v1/auth/signup").json(&json!({})).await;

    // Should return 422 for missing required fields
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Test that signup returns 422 for malformed JSON
#[tokio::test]
async fn test_signup_malformed_json_returns_bad_request() {
    let server = create_default_test_server();

    let response = server
        .post("/api/v1/auth/signup")
        .json("{ invalid json ")
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Test signup not allowing signup with existing email
#[tokio::test]
async fn test_signup_existing_email() {
    let existing = fake_data::user::model(
        uuid::Uuid::new_v4(),
        "existing_user",
        "existing_user@example.com",
    );
    let db = MockDbBuilder::new().with_query_results(vec![vec![existing]]);

    let server = create_test_server_with_db(db.build());

    let response = server
        .post("/api/v1/auth/signup")
        .json(&json!({
            "username": "newuser",
            "email": "existing_user@example.com".to_string(),
            "password": "password123".to_string()
        }))
        .await;
    assert_status_json(response, StatusCode::CONFLICT);
}

/// Test signup successful with valid data
#[tokio::test]
async fn test_signup_successful() {
    let user = fake_data::user::model(uuid::Uuid::new_v4(), "newuser", "bishoyhany006@gmail.com");
    let db = MockDbBuilder::new()
        .with_query_results(vec![
            // No existing user with email
            vec![],
            // Insert user
            vec![user.clone()],
        ])
        .with_query_results(vec![vec![fake_data::password::model_hashed(
            uuid::Uuid::new_v4(),
            user.id,
            "password123",
        )]])
        .with_query_results(vec![vec![fake_data::refresh_token::model(
            uuid::Uuid::new_v4(),
            user.id,
        )]])
        .with_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .with_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }]);
    let server = create_test_server_with_db(db.build());

    let response = server.post("/api/v1/auth/signup").json(&json!({
            "username": "newuser",
            "email": "bishoyhany006@gmail.com","password": "password123"}));

    assert_status_json(response.await, StatusCode::OK);
}
