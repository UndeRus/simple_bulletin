use core::str;

use askama::Template;
use axum::{
    extract::{Path, Query}, http::StatusCode, response::{Html, IntoResponse, Redirect}, Form
};
use axum_csrf::CsrfToken;
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::{AuthBackend, Credentials}, db, models::Advert};

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

#[derive(Template)]
#[template(path = "main.html")]
pub struct MainPageTemplate {
    adverts: Vec<Advert>,
}


#[derive(Deserialize)]
pub struct MainPageParams {
    pub before_id: Option<i64>
}

pub async fn main_board(Query(params): Query<MainPageParams>) -> impl IntoResponse {
    let adverts = if let Ok(adverts) = db::get_main_page(params.before_id).await {
        adverts
    } else {
        return "Main page error".into_response();
    };
    let template = MainPageTemplate {
        adverts
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response()).into_response()
}



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

pub async fn mod_page() -> impl IntoResponse {
    "Mod page"
}


pub async fn register(token: CsrfToken, Form(form): Form<RegisterForm>) -> impl IntoResponse {
    if let Err(_e) = token.verify(&form.csrf_token) {
        "Error".into_response()
    } else {
        // Token is valid, register
        if let Ok(_) = db::create_new_user(&form.username, &form.password).await {
            Redirect::to("/").into_response()            
        } else {
            "Failed to register".into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterFormTemplate<'a> {
    csrf_token: &'a str,
}


#[derive(Deserialize)]
pub struct RegisterForm {
    pub csrf_token: String,
    pub username: String,
    pub password: String,
}

pub async fn register_form(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();
    let template = RegisterFormTemplate {
        csrf_token: &csrf_token,
    };
    let reply_html = template.render().unwrap();

    (token, Html(reply_html)).into_response()
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


pub async fn profile(auth_session: AuthSession<AuthBackend>) -> impl IntoResponse {
    auth_session.user.unwrap().username
}