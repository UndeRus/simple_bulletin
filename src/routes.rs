use askama::Template;
use axum::{
    extract::{Path, Query}, http::StatusCode, middleware::Next, response::{Html, IntoResponse, Redirect}, Form
};
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::{AuthBackend, Credentials}, db};

#[derive(Deserialize)]
pub struct NextUrl {
    pub next: Option<String>
}

pub async fn login_with_password(
    mut auth_session: AuthSession<AuthBackend>,
    next: Query<NextUrl>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if let Some(next_url) = &next.next {
        Redirect::to(&next_url)
    } else {
        Redirect::to("/")
    }
    .into_response()
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginFormTemplate<'a> {
    name: &'a str,
}

pub async fn login_form() -> impl IntoResponse {
    let template = LoginFormTemplate {
        name: "Here is login form",
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response())
}

pub async fn logout(mut auth_session: AuthSession<AuthBackend>) -> impl IntoResponse {
    auth_session.logout().await.unwrap();
    Redirect::to("/")
}

pub async fn main_board() -> impl IntoResponse {
    "Main page"
}

pub async fn item_page(Path(item_id): Path<String>) -> impl IntoResponse {
    format!("Item page: {}", item_id)
}

pub async fn mod_page() -> impl IntoResponse {
    "Mod page"
}

pub async fn register() -> impl IntoResponse {
    "Register"
}

pub async fn register_form() -> impl IntoResponse {
    "Register form"
}

pub async fn item_new() -> impl IntoResponse {
    "New item created"
}

pub async fn item_new_form() -> impl IntoResponse {
    "Create new item form"
}
