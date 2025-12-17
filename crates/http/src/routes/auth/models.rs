use crate::config::EXPIRES_IN_JWT_ACCESS_TOKEN;
use crate::config::EXPIRES_IN_JWT_REFRESH_TOKEN;
use crate::routes::extractors::DeviceExtractor;
use chrono::Utc;
use entities::refresh_tokens;
use entities::sea_orm_active_enums::{OauthProvider, UserType};
use jsonwebtoken::Header;
use jsonwebtoken::encode;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use serde::Serialize;
use std::sync::LazyLock;
use utoipa::ToSchema;
use uuid::Uuid;

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

// pub struct UserProfile {
//     pub username: String,
//     pub bio: Option<String>,
//     pub user_type: UserType,
//     pub user_id: Uuid,
// }

#[derive(Deserialize, ToSchema)]
pub struct SeededUser {
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "SecurePassword123!")]
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginUser {
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "SecurePassword123!")]
    pub password: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Auth {
    pub refresh_token: Token,
    pub access_token: Token,
}

#[derive(Deserialize, ToSchema)]
pub struct ProviderPath {
    #[schema(example = "google")]
    pub provider: OauthProvider,
}

#[derive(Deserialize, ToSchema)]
pub struct ProviderQuery {
    pub redirect: Option<String>,
}
#[derive(Deserialize, Serialize, Clone, ToSchema)]
pub struct Token {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    #[schema(example = 1735689600)]
    pub expires_at: i64,
}

impl Token {
    pub fn new(token: String, expires_at: i64) -> Self {
        Token { token, expires_at }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_type: UserType,
    pub username: String,
    pub user_id: Uuid,
    pub exp: i64,
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl Claims {
    pub fn default(user_type: UserType, username: String, user_id: Uuid) -> Self {
        Self {
            user_type,
            username,
            user_id,
            exp: (Utc::now() + EXPIRES_IN_JWT_ACCESS_TOKEN).timestamp(),
        }
    }

    pub fn token(&self) -> Result<Token, jsonwebtoken::errors::Error> {
        // Create the authorization token
        let access_token = encode(&Header::default(), self, &KEYS.encoding);
        match access_token {
            Ok(t) => Ok(Token::new(t, self.exp)),
            Err(e) => Err(e),
        }
    }
}

impl Auth {
    pub fn new(access_token: Token, refresh_token: Token) -> Self {
        Self {
            refresh_token,
            access_token,
        }
    }

    pub fn default(
        user_type: UserType,
        username: String,
        user_id: Uuid,
        user_device_info: DeviceExtractor,
    ) -> Result<(Self, refresh_tokens::ActiveModel), jsonwebtoken::errors::Error> {
        let access_token = Claims::default(user_type, username, user_id);

        let expires_at = Utc::now() + EXPIRES_IN_JWT_REFRESH_TOKEN;
        let uuid = Uuid::new_v4();
        let ref_token = refresh_tokens::ActiveModel {
            expires_at: Set(expires_at.naive_utc()),
            user_id: Set(user_id),
            user_agent: Set(user_device_info.user_agent),
            device_type: Set(user_device_info.device_type),
            os: Set(user_device_info.os),
            engine: Set(user_device_info.engine),
            id: Set(uuid),
            ..Default::default()
        };
        Ok((
            Self::new(
                access_token.token()?,
                Token::new(uuid.to_string(), expires_at.timestamp()),
            ),
            ref_token,
        ))
    }
}

#[derive(Deserialize, Debug)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Deserialize, Debug)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

#[derive(Debug)]
pub struct OauthProfile {
    pub username: String,
    pub email: String,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct GoogleUser {
    pub sub: String, // Google ID
    pub name: String,
    pub email: String,
    pub email_verified: bool,
}
