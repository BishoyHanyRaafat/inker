#![allow(dead_code)]
//! Common fake data builders for integration tests.
//!
//! These helpers keep tests short and consistent by centralizing defaults for
//! `entities::*::{Model, ActiveModel}` values.

use chrono::{NaiveDateTime, Utc};
use sea_orm::ActiveValue::Set;
use uuid::Uuid;

pub fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub mod user {
    use super::*;
    use entities::sea_orm_active_enums::UserType;

    pub fn model(
        id: Uuid,
        username: impl Into<String>,
        email: impl Into<String>,
    ) -> entities::user::Model {
        entities::user::Model {
            id,
            username: username.into(),
            email: email.into(),
            user_type: UserType::Regular,
        }
    }

    pub fn model_with_type(
        id: Uuid,
        username: impl Into<String>,
        email: impl Into<String>,
        user_type: UserType,
    ) -> entities::user::Model {
        entities::user::Model {
            user_type,
            ..model(id, username, email)
        }
    }

    pub fn active(
        username: impl Into<String>,
        email: impl Into<String>,
    ) -> entities::user::ActiveModel {
        entities::user::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(username.into()),
            email: Set(email.into()),
            user_type: Set(UserType::Regular),
        }
    }
}

pub mod password {
    use super::*;

    /// Build a password `Model` using the *real* hasher used by the auth code.
    pub fn model_hashed(id: Uuid, user_id: Uuid, raw_password: &str) -> entities::password::Model {
        let hashed = inker_http::routes::auth::security::hash_password(raw_password)
            .expect("Failed to hash password for tests");

        entities::password::Model {
            id,
            user_id,
            content: hashed,
        }
    }

    pub fn active_hashed(user_id: Uuid, raw_password: &str) -> entities::password::ActiveModel {
        let hashed = inker_http::routes::auth::security::hash_password(raw_password)
            .expect("Failed to hash password for tests");

        entities::password::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            content: Set(hashed),
        }
    }
}

pub mod refresh_token {
    use super::*;

    pub fn model(id: Uuid, user_id: Uuid) -> entities::refresh_tokens::Model {
        entities::refresh_tokens::Model {
            id,
            user_id,
            revoked: false,
            expires_at: now(),
            device_type: None,
            os: Some("TestOS".into()),
            engine: None,
            user_agent: "TestAgent".into(),
        }
    }

    pub fn active(id: Uuid, user_id: Uuid) -> entities::refresh_tokens::ActiveModel {
        entities::refresh_tokens::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            revoked: Set(false),
            expires_at: Set(now()),
            device_type: Set(None),
            os: Set(Some("TestOS".into())),
            engine: Set(None),
            user_agent: Set("TestAgent".into()),
        }
    }
}

pub mod note {
    use super::*;

    pub fn model(id: Uuid, user_id: Uuid, title: impl Into<String>) -> entities::note::Model {
        let t = title.into();
        let ts = now();
        entities::note::Model {
            id,
            title: t,
            user_id,
            created_at: ts,
            updated_at: ts,
            deleted_at: None,
            archived: false,
            pinned: false,
        }
    }
}

pub mod block {
    use super::*;
    use serde_json::json;

    pub fn model_text(
        id: Uuid,
        note_id: Uuid,
        order: f32,
        version: i32,
        text: &str,
    ) -> entities::block::Model {
        entities::block::Model {
            id,
            note_id,
            order,
            version,
            content: json!({
                "type": "text",
                "data": {
                    "segments": [
                        { "text": text, "marks": [] }
                    ]
                }
            }),
        }
    }
}
