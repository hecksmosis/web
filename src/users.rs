use axum::{
    extract::Path,
    response::{Html, IntoResponse, Redirect},
    Extension, Form,
};
use tera::Context;

use crate::{
    auth::{get_user, is_logged_in_user, AuthState},
    errors::{NoUser, NotAdmin, NotLoggedIn},
    utils::error_page,
    Database, Templates,
};

async fn get_users(database: &Database) -> Vec<String> {
    const QUERY: &str = "SELECT username FROM users LIMIT 100";

    sqlx::query_as(QUERY)
        .fetch_all(database)
        .await
        .unwrap()
        .into_iter()
        .map(|(value,)| value)
        .collect()
}

async fn get_admins(database: &Database) -> Vec<String> {
    const QUERY: &str = "SELECT username FROM users WHERE permission_level = 1 LIMIT 100";

    sqlx::query_as(QUERY)
        .fetch_all(database)
        .await
        .unwrap()
        .into_iter()
        .map(|(value,)| value)
        .collect()
}

pub(crate) async fn users(
    Extension(database): Extension<Database>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    let users = get_users(&database).await;

    let mut context = Context::new();
    context.insert("users", &users);

    Html(templates.render("users", &context).unwrap())
}

pub(crate) async fn profile(
    Extension(mut current_user): Extension<AuthState>,
    Extension(database): Extension<Database>,
    Form(ProfileForm { profile }): Form<ProfileForm>,
) -> impl IntoResponse {
    if !current_user.logged_in() {
        return Err(error_page(&NotLoggedIn));
    }

    let user = current_user.get_user().await.unwrap();

    const QUERY: &str = "UPDATE users SET profile = $1 WHERE username = $2;";

    sqlx::query(QUERY)
        .bind(&profile)
        .bind(&user.username)
        .execute(&database)
        .await
        .unwrap();

    Ok(Redirect::to("/me"))
}

pub(crate) async fn user(
    Path(username): Path<String>,
    Extension(mut auth_state): Extension<AuthState>,
    Extension(database): Extension<Database>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    if let Some((username, profile, permission_level)) = get_user(&username, &database).await {
        let user_is_self = is_logged_in_user(&mut auth_state, &username).await;

        let _ = PermissionLevel::from(permission_level);
        // TODO: Add admin page

        let mut context = Context::new();
        context.insert("username", &username);
        context.insert("is_self", &user_is_self);
        if profile.is_none() {
            context.insert("profile", &"No profile set");
        } else {
            context.insert("profile", &profile.unwrap());
        }
        Ok(Html(templates.render("user", &context).unwrap()))
    } else {
        Err(error_page(&NoUser(username)))
    }
}

pub(crate) async fn me(
    Extension(mut current_user): Extension<AuthState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Some(user) = current_user.get_user().await {
        Ok(Redirect::to(&format!("/user/{}", user.username)))
    } else {
        Err(error_page(&NotLoggedIn))
    }
}

pub(crate) async fn admin(
    Extension(mut auth_state): Extension<AuthState>,
    Extension(database): Extension<Database>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    if auth_state.is_admin().await {
        let current_username = &auth_state.get_user().await.unwrap().username;
        let users = get_users(&database).await;
        let admins = get_admins(&database)
            .await
            .iter()
            .filter(|&n| n != current_username)
            .map(String::from)
            .collect::<Vec<_>>();
        let mut context = Context::new();
        context.insert("users", &users);
        context.insert("admins", &admins);
        Ok(Html(templates.render("admin", &context).unwrap()))
    } else {
        Err(error_page(&NotAdmin))
    }
}

pub(crate) async fn add_admin(
    Path(username): Path<String>,
    Extension(database): Extension<Database>,
    Extension(mut auth_state): Extension<AuthState>,
) -> impl IntoResponse {
    if auth_state.is_admin().await {
        let user = get_user(&username, &database).await;
        if let Some((_, _, permission_level)) = user {
            if permission_level == 0 {
                const QUERY: &str = "UPDATE users SET permission_level = 1 WHERE username = $1;";

                sqlx::query(QUERY)
                    .bind(&username)
                    .execute(&database)
                    .await
                    .unwrap();
            }
        }
        Ok(Redirect::to("/admin"))
    } else {
        Err(error_page(&NotAdmin))
    }
}

pub(crate) async fn remove_admin(
    Path(username): Path<String>,
    Extension(database): Extension<Database>,
    Extension(mut auth_state): Extension<AuthState>,
) -> impl IntoResponse {
    if auth_state.is_admin().await {
        let user = get_user(&username, &database).await;
        if let Some((_, _, permission_level)) = user {
            if permission_level == 1 {
                const QUERY: &str = "UPDATE users SET permission_level = 0 WHERE username = $1;";

                sqlx::query(QUERY)
                    .bind(&username)
                    .execute(&database)
                    .await
                    .unwrap();
            }
        }
        Ok(Redirect::to("/admin"))
    } else {
        Err(error_page(&NotAdmin))
    }
}

#[derive(serde::Deserialize)]
pub struct ProfileForm {
    profile: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionLevel {
    User,
    Admin,
}

impl From<PermissionLevel> for i32 {
    fn from(permission_level: PermissionLevel) -> Self {
        match permission_level {
            PermissionLevel::User => 0,
            PermissionLevel::Admin => 1,
        }
    }
}

impl From<i32> for PermissionLevel {
    fn from(permission_level: i32) -> Self {
        match permission_level {
            0 => PermissionLevel::User,
            1 => PermissionLevel::Admin,
            _ => panic!("Invalid permission level"),
        }
    }
}
