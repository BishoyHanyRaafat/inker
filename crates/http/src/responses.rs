use super::config::DEBUG;
use axum::response::{IntoResponse, Json};
use serde::Serialize;
use std::{collections::HashMap, fmt::Debug};
use tracing::trace;
use utoipa::ToSchema;

fn skip_in_production(opt: &Option<String>) -> bool {
    !*DEBUG || opt.is_none()
}

#[derive(Serialize, ToSchema, Debug)]
#[schema(as = Response, description = "Standard API Response")]
pub struct ApiResponse<T> {
    #[schema(example = true)]
    success: bool,
    #[schema(example = "Operation completed successfully")]
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "https://example.com/redirect")]
    redirect_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "skip_in_production")]
    debug: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[schema(example = json!({"field": ["error message"]}))]
    errors: HashMap<String, Vec<String>>,
    #[serde(skip_serializing)]
    status: axum::http::StatusCode,
}

impl<T> ApiResponse<T> {
    pub fn error_response(status: axum::http::StatusCode, message: String) -> Self {
        ApiResponse {
            success: false,
            message,
            data: None,
            debug: None,
            errors: HashMap::new(),
            status,
            redirect_to: None,
        }
    }

    pub fn with_error_detail(mut self, field: &str, error: &str) -> Self {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(error.to_string());
        self
    }

    pub fn success_response(status: axum::http::StatusCode, message: &str, data: T) -> Self {
        ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
            debug: None,
            errors: HashMap::new(),
            status,
            redirect_to: None,
        }
    }

    pub fn with_debug(mut self, debug: impl Debug) -> Self {
        self.debug = Some(format!("{:?}", debug));
        self
    }

    #[allow(dead_code)]
    pub fn redirect(mut self, path: String) -> Self {
        self.redirect_to = Some(path);
        self
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        if self.success {
            if let Some(ref data) = self.data {
                trace!(
                    "  Response data: {:?}",
                    serde_json::to_string(data)
                        .unwrap_or_else(|_| "Unable to serialize data".to_string())
                );
            }
        } else if !self.errors.is_empty() {
            trace!("  Error details: {:?}", self.errors);
        }

        (self.status, Json(self)).into_response()
    }
}
