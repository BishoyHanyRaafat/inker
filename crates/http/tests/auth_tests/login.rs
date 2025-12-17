use crate::common::{
    MockDbBuilder, assert_status_json, create_default_test_server, create_test_server_with_db,
    setup_test_env,
};
use crate::fake_data;
use axum::http::StatusCode;
use sea_orm::MockExecResult;
use serde_json::json;

/// Test that login returns 400 when email is empty
#[tokio::test]
async fn test_login_empty_email_returns_bad_request() {
    let server = create_default_test_server();

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "",
            "password": "somepassword"
        }))
        .await;

    // Should return 400 Bad Request for missing/empty email
    let body = assert_status_json(response, StatusCode::BAD_REQUEST);
    assert!(
        body["success"] == false,
        "response body:\n{}",
        serde_json::to_string_pretty(&body).unwrap()
    );
}

/// Test that login returns 400 when password is empty
#[tokio::test]
async fn test_login_empty_password_returns_bad_request() {
    let server = create_default_test_server();

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": ""
        }))
        .await;

    // Should return 400 Bad Request for missing/empty password
    let body = assert_status_json(response, StatusCode::BAD_REQUEST);
    assert!(
        body["success"] == false,
        "response body:\n{}",
        serde_json::to_string_pretty(&body).unwrap()
    );
}

/// Test that login returns 401 for non-existent user
#[tokio::test]
async fn test_login_nonexistent_user_returns_unauthorized() {
    // Create mock database that returns empty result for user query
    let db = MockDbBuilder::new()
        .with_query_results::<entities::user::Model>(vec![vec![]]) // No user found
        .build();

    setup_test_env();
    let server = create_test_server_with_db(db);

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "somepassword123"
        }))
        .await;

    // Should return 401 Unauthorized for non-existent user
    let body = assert_status_json(response, StatusCode::UNAUTHORIZED);
    assert!(
        body["success"] == false,
        "response body:\n{}",
        serde_json::to_string_pretty(&body).unwrap()
    );
}

/// Test that login returns 422 when JSON body is missing required fields
#[tokio::test]
async fn test_login_missing_fields_returns_unprocessable() {
    let server = create_default_test_server();

    // Send incomplete JSON (missing password field)
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "test@example.com"
        }))
        .await;

    // Should return 422 Unprocessable Entity for invalid JSON structure
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Test that the auth routes are properly mounted
#[tokio::test]
async fn test_auth_routes_exist() {
    let server = create_default_test_server();

    // Test that login route exists (POST method)
    let response = server.post("/api/v1/auth/login").json(&json!({})).await;
    // 422 means the route exists but JSON is invalid (not 404)
    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);

    // Test that signup route exists (POST method)
    let response = server.post("/api/v1/auth/signup").json(&json!({})).await;
    assert_ne!(response.status_code(), StatusCode::NOT_FOUND);
}

/// Test that login route rejects GET requests
#[tokio::test]
async fn test_login_rejects_get_method() {
    let server = create_default_test_server();

    let response = server.get("/api/v1/auth/login").await;

    // Should return 405 Method Not Allowed for GET on login
    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
}

/// Test that login returns 422 for completely empty JSON body
#[tokio::test]
async fn test_login_empty_json_returns_unprocessable() {
    let server = create_default_test_server();

    let response = server.post("/api/v1/auth/login").json(&json!({})).await;

    // Should return 422 for missing required fields
    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Test that login returns 422 for malformed JSON
#[tokio::test]
async fn test_login_malformed_json_returns_bad_request() {
    let server = create_default_test_server();

    let response = server
        .post("/api/v1/auth/login")
        .content_type("application/json")
        .bytes("{ invalid json }".as_bytes().into())
        .await;

    // Should return 400 Bad Request for malformed JSON
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_successful_with_password() {
    let user = fake_data::user::model(
        uuid::Uuid::new_v4(),
        "bishoyhany",
        "bishoyhany006@gmail.com",
    );
    let password = fake_data::password::model_hashed(uuid::Uuid::new_v4(), user.id, "password123");
    let existing_refresh = fake_data::refresh_token::model(uuid::Uuid::new_v4(), user.id);

    let db = MockDbBuilder::new()
        .with_tuple_query_results(vec![vec![(user.clone(), Some(password.clone()))]])
        .with_query_results(vec![vec![existing_refresh]])
        .with_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .build();

    let server = create_test_server_with_db(db);

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": user.email,
            "password": "password123"
        }))
        .await;
    assert_status_json(response, StatusCode::OK);
}

#[tokio::test]
async fn test_login_wrong_password_returns_unauthorized() {
    let user = fake_data::user::model(
        uuid::Uuid::new_v4(),
        "bishoyhany",
        "bishoyhany006@gmail.com",
    );
    let password =
        fake_data::password::model_hashed(uuid::Uuid::new_v4(), user.id, "correct_password");

    let db = MockDbBuilder::new()
        .with_tuple_query_results(vec![vec![(user.clone(), Some(password.clone()))]])
        .with_query_results::<entities::refresh_tokens::Model>(vec![vec![]])
        .build();

    let server = create_test_server_with_db(db);

    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": user.email,
            "password": "wrong_password"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}
