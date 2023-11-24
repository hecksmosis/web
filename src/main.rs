mod auth;
mod errors;
mod users;
mod utils;

use shuttle_runtime::CustomError;
use std::sync::{Arc, Mutex};
use users::{me, profile, user, users, admin, add_admin, remove_admin};

use axum::{
    extract::Extension,
    http::{self, Response},
    middleware,
    response::{Html, IntoResponse},
    routing::{any, get, post},
    Form, Router,
};

use auth::{auth, delete_user, login, signup, AuthState};
use errors::{NotLoggedIn, SignupError};
use pbkdf2::password_hash::rand_core::OsRng;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use shuttle_axum::ShuttleAxum;
use sqlx::{Executor, PgPool};
use tera::{Context, Tera};
use utils::*;

type Templates = Arc<Tera>;
type Database = sqlx::PgPool;
type Random = Arc<Mutex<ChaCha8Rng>>;

const USER_COOKIE_NAME: &str = "user_token";
const COOKIE_MAX_AGE: &str = "9999999";

#[shuttle_runtime::main]
async fn server(#[shuttle_shared_db::Postgres] pool: PgPool) -> ShuttleAxum {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    Ok(get_router(pool).into())
}

pub fn get_router(database: Database) -> Router {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../templates/base.html")),
        ("admin", include_str!("../templates/admin.html")),
        ("index", include_str!("../templates/index.html")),
        ("signup", include_str!("../templates/signup.html")),
        ("login", include_str!("../templates/login.html")),
        ("users", include_str!("../templates/users.html")),
        ("user", include_str!("../templates/user.html")),
    ])
    .unwrap();

    let middleware_database = database.clone();
    let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());

    Router::new()
        .route("/", get(index))
        .route("/signup", get(get_signup).post(post_signup))
        .route("/login", get(get_login).post(post_login))
        .route("/logout", post(logout_response))
        .route("/delete", post(post_delete))
        .route("/me", get(me))
        .route("/user/:username", get(user))
        .route("/profile", post(profile))
        .route("/users", get(users))
        .route("/admin", get(admin))
        .route("/admin/add/:username", post(add_admin))
        .route("/admin/remove/:username", post(remove_admin))
        .route("/styles.css", any(styles))
        .layer(middleware::from_fn(move |req, next| {
            auth(req, next, middleware_database.clone())
        }))
        .layer(Extension(Arc::new(tera)))
        .layer(Extension(database))
        .layer(Extension(Arc::new(Mutex::new(random))))
}

async fn index(
    Extension(current_user): Extension<AuthState>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("logged_in", &current_user.logged_in());
    context.insert("home_screen", &true);
    Html(templates.render("index", &context).unwrap())
}

async fn get_signup(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("signup", &Context::new()).unwrap())
}

async fn get_login(Extension(templates): Extension<Templates>) -> impl IntoResponse {
    Html(templates.render("login", &Context::new()).unwrap())
}

async fn post_signup(
    Extension(database): Extension<Database>,
    Extension(random): Extension<Random>,
    Form(SignupForm {
        username,
        password,
        confirm_password,
    }): Form<SignupForm>,
) -> impl IntoResponse {
    if password != confirm_password {
        return Err(error_page(&SignupError::PasswordsDoNotMatch));
    }

    if password.len() < 8 {
        return Err(error_page(&SignupError::InvalidPassword));
    }

    match signup(&database, random, &username, &password).await {
        Ok(session_token) => Ok(login_response(session_token)),
        Err(error) => Err(error_page(&error)),
    }
}

async fn post_login(
    Extension(database): Extension<Database>,
    Extension(random): Extension<Random>,
    Form(LoginForm { username, password }): Form<LoginForm>,
) -> impl IntoResponse {
    match login(&database, random, username, password).await {
        Ok(session_token) => Ok(login_response(session_token)),
        Err(err) => Err(error_page(&err)),
    }
}

async fn post_delete(Extension(current_user): Extension<AuthState>) -> impl IntoResponse {
    if !current_user.logged_in() {
        return Err(error_page(&NotLoggedIn));
    }

    delete_user(current_user).await;

    Ok(logout_response().await)
}

async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../public/styles.css").to_owned())
        .unwrap()
}

#[derive(serde::Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct SignupForm {
    username: String,
    password: String,
    confirm_password: String,
}
