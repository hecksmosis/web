use crate::{auth::SessionToken, COOKIE_MAX_AGE, USER_COOKIE_NAME, errors::ErrorInfo};
use axum::{
    body::Empty,
    http::{Response, StatusCode},
    response::IntoResponse,
};

pub(crate) fn login_response(session_token: SessionToken) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age={}",
                USER_COOKIE_NAME,
                session_token.into_cookie_value(),
                COOKIE_MAX_AGE
            ),
        )
        .body(Empty::new())
        .unwrap()
}

pub(crate) async fn logout_response() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header("Set-Cookie", format!("{}=_; Max-Age=0", USER_COOKIE_NAME,))
        .body(Empty::new())
        .unwrap()
}

pub(crate) fn error_page(err: &dyn ErrorInfo) -> impl IntoResponse {
    Response::builder()
        .status(err.error_info().0)
        .body(err.error_info().1)
        .unwrap()
}
