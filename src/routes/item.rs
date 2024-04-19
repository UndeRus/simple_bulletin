use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_csrf::CsrfToken;
use axum_login::{AuthSession, AuthzBackend};
use serde::Deserialize;

use crate::{
    auth::{AuthBackend, AuthPermission},
    db,
    models::Advert,
};

#[derive(Template)]
#[template(path = "item.html")]
pub struct ItemPageTemplate {
    csrf_token: String,
    advert: Advert,
    own_advert: bool,
}

#[derive(Deserialize)]
pub struct ItemEditForm {
    csrf_token: String
}

pub async fn item_page_edit(
    token: CsrfToken,    
    auth_session: AuthSession<AuthBackend>,
    Path(advert_id): Path<i64>,
    Form(form): Form<ItemEditForm>,
) -> impl IntoResponse {
    if token.verify(&form.csrf_token).is_err() {
        return "Failed to get csrf token".into_response();
    };

    let user_id = if let Some(user_id) = auth_session.user.map(|u|u.id) {
        user_id
    } else {
        return "No user found".into_response();
    };

    if let Ok(is_own_advert) = db::check_advert_belong_to_user(user_id, advert_id).await {
        if is_own_advert {
            //TODO: unpublish
            if db::toggle_advert_publish(advert_id, false).await.is_ok() {
                return Redirect::to(&format!("/item/{}", advert_id)).into_response();
            } else {
                return "Failed to unpublish advert".into_response();
            }
        } else {
            return "You tried edit someone else advert".into_response();    
        }
    } else {
        return "You tried edit someone else advert".into_response();
    }
}

pub async fn item_page(
    token: CsrfToken,
    auth_session: AuthSession<AuthBackend>,
    Path(item_id): Path<i64>,
) -> impl IntoResponse {
    let csrf_token = if let Ok(token) = token.authenticity_token() {
        token
    } else {
        return "Failed to get csrf token".into_response();
    };

    let user = auth_session.user.clone();
    let user_id = user.clone().map(|u|u.id);
    let is_admin = if let Some(user) = user {
        auth_session
            .backend
            .has_perm(
                &user,
                AuthPermission {
                    name: "admin.read".to_string(),
                },
            )
            .await
            .unwrap_or(false)
    } else {
        false
    };
    let (advert, own_advert) = if let Ok(advert) =
        db::get_advert_by_id(user_id, item_id, is_admin).await
    {
        advert
    } else {
        return "Not found".into_response();
    };

    let template = ItemPageTemplate {
        csrf_token,
         advert ,
         own_advert,
    };
    let reply_html = template.render().unwrap();
    (token, Html(reply_html).into_response()).into_response()
}

#[derive(Deserialize)]
pub struct ItemNewForm {
    pub title: String,
    pub content: String,
    pub csrf_token: String,
}

pub async fn item_new(
    token: CsrfToken,
    auth_session: AuthSession<AuthBackend>,
    Form(form): Form<ItemNewForm>,
) -> impl IntoResponse {
    let user = if let Some(user) = auth_session.user {
        user
    } else {
        return "User not found".into_response();
    };

    if let Err(_e) = token.verify(&form.csrf_token) {
        return "Error".into_response();
    }
    let new_advert_id =
        if let Ok(id) = db::create_new_advert(user.id, &form.title, &form.content).await {
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
    let csrf_token = if let Ok(csrf_token) = token.authenticity_token() {
        csrf_token
    } else {
        return "Failed to get csrf token".into_response();
    };
    let template = ItemNewFormTemplate {
        csrf_token: &csrf_token,
    };
    let reply_html = template.render().unwrap();
    (token, Html(reply_html)).into_response()
}
