use crate::errors::{AppError, AppErrorTrait, AuthError, BaseError};
use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
    password_hash::{PasswordHash, SaltString},
};
use password_hash::rand_core::OsRng;
use regex::Regex;
use std::sync::LazyLock;

pub const MIN_PASSWORD_LEN: usize = 8;
pub static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^[a-z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$"
    )
    .expect("hard-coded e-mail regex must compile")
});
pub fn validate_user_payload(email: &str, username: &str, password: &str) -> Result<(), AppError> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(AuthError::ValidationError
            .get_response()
            .with_error_detail("email", "not a valid address"));
    }

    if password.len() < MIN_PASSWORD_LEN {
        return Err(AuthError::ValidationError
            .get_response()
            .with_error_detail("password", "too short"));
    }

    if password.contains(username) {
        return Err(AuthError::ValidationError
            .get_response()
            .with_error_detail("password", "must not contain the username"));
    }

    Ok(())
}

fn argon2() -> Argon2<'static> {
    let params = Params::new(64 * 1_024, 3, 1, None).expect("invalid Argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    argon2()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))
        .map(|hash| hash.to_string())
}

pub fn verify_password(stored_hash: &str, password: &str) -> Result<(), AppError> {
    let parsed =
        PasswordHash::new(stored_hash).map_err(|_| AuthError::WrongCredentials.get_response())?;

    argon2()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AuthError::WrongCredentials.get_response())
}

pub fn validate_csrf(state_csrf: Option<&str>, cookie_csrf: &str) -> Result<(), AppError> {
    match state_csrf {
        Some(csrf_token) if csrf_token == cookie_csrf => Ok(()),
        Some(_) => Err(AuthError::InvalidToken
            .get_response()
            .with_debug("Invalid CSRF token".to_string())),
        None => Err(AuthError::InvalidToken
            .get_response()
            .with_debug("Missing CSRF token in state param".to_string())),
    }
}

pub fn extract_redirect_and_csrf(state: &str) -> (Option<&str>, Option<&str>) {
    let mut csrf = None;
    let mut redirect = None;

    for pair in state.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "csrf" => csrf = Some(v),
                "redirect" => redirect = Some(v),
                _ => {}
            }
        }
    }
    (csrf, redirect)
}
