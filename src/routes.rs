use askama::Template;
use axum::{
    extract::{Path, Query}, http::StatusCode, response::{Html, IntoResponse, Redirect}, Form
};
use axum_csrf::CsrfToken;
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
pub struct LoginFormTemplate {

}

pub async fn login_form() -> impl IntoResponse {
    let template = LoginFormTemplate {
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




pub async fn register(token: CsrfToken, Form(form): Form<RegisterForm>) -> impl IntoResponse {
    if let Err(e) = token.verify(&form.csrf_token) {
        // Wrong csrf
        "Token is invalid"
    } else {
        // Token is valid, register
        "Token is Valid lets do stuff!"
    }

}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterFormTemplate<'a> {
    csrf_token: &'a str,
}


#[derive(Deserialize)]
pub struct RegisterForm {
    pub csrf_token: String
}

pub async fn register_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();
    let template = RegisterFormTemplate {
        csrf_token: &csrf_token,
    };
    let reply_html = template.render().unwrap();

    (token, Html(reply_html)).into_response()
}

pub async fn item_new() -> impl IntoResponse {
    "New item created"
}

pub async fn item_new_form() -> impl IntoResponse {
    "Create new item form"
}
