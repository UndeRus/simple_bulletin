use askama::Template;
use axum::{
    extract::{Path, Query}, http::StatusCode, response::{Html, IntoResponse, Redirect}, Form
};
use axum_csrf::CsrfToken;
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::{AuthBackend, Credentials}, db, models::Advert};


#[derive(Template)]
#[template(path = "item.html")]
pub struct ItemPageTemplate {
    advert: Advert,
}

pub async fn item_page(Path(item_id): Path<i64>) -> impl IntoResponse {
    let advert = if let Ok(advert)  = db::get_advert_by_id(item_id, true).await {
        advert
    } else {
        return "Not found".into_response();
    };

    let template = ItemPageTemplate {
        advert
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response()).into_response()
}

#[derive(Deserialize)]
pub struct ItemNewForm {
    pub title: String,
    pub content: String,
    pub csrf_token: String,
}

pub async fn item_new(token: CsrfToken, auth_session: AuthSession<AuthBackend>, Form(form): Form<ItemNewForm>) -> impl IntoResponse {
    let user = if let Some(user) = auth_session.user {
        user
    } else {
        return "User not found".into_response();
    };

    if let Err(_e) = token.verify(&form.csrf_token) {
        return "Error".into_response();
    }
    let new_advert_id = if let Ok(id) = db::create_new_advert(user.id, &form.title, &form.content).await {
        id
    } else {
        return "Failed to create advert".into_response();
    };

    Redirect::to(&format!("/item/{}", new_advert_id)).into_response()
}

#[derive(Template)]
#[template(path = "item_new.html")]
pub struct ItemNewFormTemplate<'a> {
    pub csrf_token: &'a str,
}

pub async fn item_new_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();
    let template = ItemNewFormTemplate {
        csrf_token: &csrf_token,
    };
    let reply_html = template.render().unwrap();
    (token, Html(reply_html)).into_response()
}
