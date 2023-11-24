use std::{error::Error, fmt::Display};
use axum::http::StatusCode;

pub trait ErrorInfo {
    fn error_info(&self) -> (StatusCode, String);
}

#[derive(Debug)]
pub(crate) struct NotLoggedIn;

impl Display for NotLoggedIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Not logged in")
    }
}

impl Error for NotLoggedIn {}

impl ErrorInfo for NotLoggedIn {
    fn error_info(&self) -> (StatusCode, String) {
        (StatusCode::UNAUTHORIZED, self.to_string())
    }
}

#[derive(Debug)]
pub(crate) struct NotAdmin;

impl Display for NotAdmin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Not an administator")
    }
}

impl Error for NotAdmin {}

impl ErrorInfo for NotAdmin {
    fn error_info(&self) -> (StatusCode, String) {
        (StatusCode::UNAUTHORIZED, self.to_string())
    }
}

#[derive(Debug)]
pub(crate) enum SignupError {
    UsernameExists,
    InvalidUsername,
    PasswordsDoNotMatch,
    InvalidPassword,
    InternalError,
}

impl Display for SignupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignupError::InvalidUsername => f.write_str("Invalid username"),
            SignupError::UsernameExists => f.write_str("Username already exists"),
            SignupError::PasswordsDoNotMatch => f.write_str("Passwords do not match"),
            SignupError::InvalidPassword => f.write_str("Invalid Password"),
            SignupError::InternalError => f.write_str("Internal Error"),
        }
    }
}

impl Error for SignupError {}

impl ErrorInfo for SignupError {
    fn error_info(&self) -> (StatusCode, String) {
        match self {
            SignupError::InvalidUsername => (StatusCode::BAD_REQUEST, self.to_string()),
            SignupError::UsernameExists => (StatusCode::BAD_REQUEST, self.to_string()),
            SignupError::PasswordsDoNotMatch => (StatusCode::BAD_REQUEST, self.to_string()),
            SignupError::InvalidPassword => (StatusCode::BAD_REQUEST, self.to_string()),
            SignupError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum LoginError {
    UserDoesNotExist,
    WrongPassword,
}

impl Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::UserDoesNotExist => f.write_str("User does not exist"),
            LoginError::WrongPassword => f.write_str("Wrong password"),
        }
    }
}

impl Error for LoginError {}

impl ErrorInfo for LoginError {
    fn error_info(&self) -> (StatusCode, String) {
        match self {
            LoginError::UserDoesNotExist => (StatusCode::BAD_REQUEST, self.to_string()),
            LoginError::WrongPassword => (StatusCode::UNAUTHORIZED, self.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct NoUser(pub String);

impl Display for NoUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("could not find user '{}'", self.0))
    }
}

impl Error for NoUser {}

impl ErrorInfo for NoUser {
    fn error_info(&self) -> (StatusCode, String) {
        (StatusCode::NOT_FOUND, self.to_string())
    }
}