use super::models::{Auth, OauthProfile};
use super::security::{hash_password, verify_password};
use crate::errors::{AppErrorTrait, BaseError};
use crate::routes::extractors::DeviceExtractor;
use crate::{
    AppState,
    errors::{AppError, AuthError},
};
use async_trait::async_trait;
use chrono::Utc;
use entities::sea_orm_active_enums::{OauthProvider, TokenType};
use entities::{password, user};
use oauth2::basic::BasicTokenType;
use oauth2::{EmptyExtraTokenFields, StandardTokenResponse, TokenResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

use entities::{prelude::*, refresh_tokens};

pub trait PasswordAuthenticatable {
    fn verify(&self, raw: &str) -> Result<(), AppError>;
    fn hash(raw: &str) -> Result<String, AppError>;
}

impl PasswordAuthenticatable for password::Model {
    fn verify(&self, raw: &str) -> Result<(), AppError> {
        verify_password(self.content.as_ref(), raw)
    }

    fn hash(raw: &str) -> Result<String, AppError> {
        hash_password(raw)
    }
}

pub trait TokenGeneratable {
    fn generate_tokens(
        &self,
        device: DeviceExtractor,
    ) -> Result<(Auth, refresh_tokens::ActiveModel), AppError>;
}

#[async_trait]
pub trait DatabaseInsertable: Sized + Send + 'static {
    async fn insert_refresh_token(
        self,
        state: &AppState,
    ) -> Result<refresh_tokens::Model, AppError>;
}

#[async_trait]
impl DatabaseInsertable for refresh_tokens::ActiveModel {
    async fn insert_refresh_token(
        self,
        state: &AppState,
    ) -> Result<refresh_tokens::Model, AppError> {
        self.insert(&state.db)
            .await
            .map_err(|e| AuthError::TokenCreation.get_response().with_debug(e))
    }
}

impl TokenGeneratable for user::Model {
    fn generate_tokens(
        &self,
        device: DeviceExtractor,
    ) -> Result<(Auth, refresh_tokens::ActiveModel), AppError> {
        Auth::default(
            self.user_type.clone(),
            self.username.clone(),
            self.id,
            device,
        )
        .map_err(|_| AuthError::TokenCreation.get_response())
    }
}

pub async fn find_or_create_user(
    state: &AppState,
    profile: &OauthProfile,
    provider: OauthProvider,
    scopes: Vec<String>,
    token_result: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
) -> Result<user::Model, AppError> {
    let access_token = token_result.access_token().secret();
    let refresh_token = token_result.refresh_token();
    let expires_in = token_result.expires_in();
    let token_type = match token_result.token_type() {
        BasicTokenType::Bearer => TokenType::Bearer,
        BasicTokenType::Mac => TokenType::Mac,
        BasicTokenType::Extension(_) => TokenType::Extension,
    };

    let email_to_lookup = profile.email.clone();

    let optional_usr: Option<(user::Model, Option<entities::oauth2::Model>)> = User::find()
        .filter(user::Column::Email.eq(email_to_lookup.clone()))
        .find_also_related(entities::oauth2::Entity)
        .one(&state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    match optional_usr {
        Some((usr, Some(acc))) => {
            if acc.oauth_provider != provider {
                // User exists with a different OAuth provider - tell them to use the correct one
                Err(
                    AuthError::InvalidProvider(format!(
                        "This email is already registered with {:?}. Please sign in using {:?} instead.",
                        acc.oauth_provider, acc.oauth_provider
                    ))
                    .get_response(),
                )
            } else {
                Ok(usr)
            }
        }
        Some((_, None)) => {
            // User exists but signed up with email/password (no OAuth record)
            Err(AuthError::InvalidProvider(
                "This email is registered with password login. Please use email and password to sign in.".into()
            ).get_response())
        }
        None => {
            let usr = user::ActiveModel {
                username: Set(profile.username.clone()),
                email: Set(profile.email.clone()),
                ..Default::default()
            };
            let txn: DatabaseTransaction = state.db.begin().await?;
            let inserted_user = usr
                .insert(&txn)
                .await
                .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;
            let oauth = entities::oauth2::ActiveModel {
                user_id: Set(inserted_user.id),
                oauth_provider: Set(provider),
                provider_user_id: Set(profile.id.clone()),
                access_token: Set(access_token.to_owned()),
                refresh_token: Set(refresh_token.map(|r| r.secret().to_owned())),
                expires_at: Set(expires_in.map(|d| (Utc::now() + d).naive_utc())),
                scopes: Set(scopes.into()),
                oauth_type: Set(token_type),
                ..Default::default()
            };
            oauth
                .insert(&txn)
                .await
                .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;
            txn.commit()
                .await
                .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;
            Ok(inserted_user)
        }
    }
}
