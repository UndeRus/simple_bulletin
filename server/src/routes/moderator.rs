use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    Form,
};
use axum_csrf::CsrfToken;
use serde::Deserialize;

use crate::{auth_models::User, db_orm, models::Advert, AppState};

const ADVERTS_LIMIT: i64 = 10;
const USERS_LIMIT: i64 = 10;

const ACTIVATE_USER_ACTION: &'static str = "au";
const DEACTIVATE_USER_ACTION: &'static str = "du";
const PUBLISH_ADVERT_ACTION: &'static str = "pa";
const UNPUBLISH_ADVERT_ACTION: &'static str = "ua";

#[derive(Deserialize)]
pub struct ModPageParams {
    user_page: Option<i64>,
    advert_page: Option<i64>,
}

#[derive(Template)]
#[template(path = "moderator.html")]
struct ModeratorPageTemplate {
    csrf_token: String,

    adverts: Vec<Advert>,
    users: Vec<User>,

    advert_page: i64,
    total_advert_pages: i64,
    user_page: i64,
    total_user_pages: i64,

    logged_in: bool,
}

pub async fn mod_page(
    State(state): State<AppState>,
    token: CsrfToken,
    Query(params): Query<ModPageParams>,
) -> impl IntoResponse {
    let csrf_token = if let Ok(csrf_token) = token.authenticity_token() {
        csrf_token
    } else {
        return "Failed to get csrf token".into_response();
    };

    let advert_page = params.advert_page.unwrap_or(1);
    let user_page = params.user_page.unwrap_or(1);
    let adverts_per_page = ADVERTS_LIMIT;
    let users_per_page = USERS_LIMIT;

    let db = state.db.read().await;

    let ((adverts, total_advert_pages), (users, total_user_pages)) = if let Ok((adverts, users)) =
        db_orm::get_mod_page(
            &db,
            (advert_page - 1) as u64,
            adverts_per_page,
            (user_page - 1) as u64,
            users_per_page,
        )
        .await
    {
        (adverts, users)
    } else {
        return "Failed to load mod page info".into_response();
    };

    let template = ModeratorPageTemplate {
        csrf_token,
        adverts,
        users,
        advert_page,
        total_advert_pages,
        user_page,
        total_user_pages,
        logged_in: true,
    };
    let reply_html = template.render().unwrap();
    (token, Html(reply_html).into_response()).into_response()
}

#[derive(Deserialize)]
#[serde(tag = "action")]
pub enum ModAction {
    ActivateUser,
    DeactivateUser,
    PublishAdvert,
    UnpublishAdvert,
}

#[derive(Deserialize)]

pub struct ModEditForm {
    csrf_token: String,
    action: String,
    id: i64,
}

pub async fn mod_edit(
    State(state): State<AppState>,
    token: CsrfToken,
    Query(params): Query<ModPageParams>,
    Form(form): Form<ModEditForm>,
) -> impl IntoResponse {
    if token.verify(&form.csrf_token).is_err() {
        return "Failed to verify csrf".into_response();
    }
    let db = state.db.write().await;

    let result = match form.action.as_str() {
        ACTIVATE_USER_ACTION => db_orm::toggle_user_active(&db, form.id, true).await,
        DEACTIVATE_USER_ACTION => db_orm::toggle_user_active(&db, form.id, false).await,
        PUBLISH_ADVERT_ACTION => db_orm::toggle_advert_publish(&db, form.id, true).await,
        UNPUBLISH_ADVERT_ACTION => db_orm::toggle_advert_publish(&db, form.id, false).await,
        _ => Err(()),
    };
    if result.is_ok() {
        Redirect::to(&format!(
            "/mod?advert_page={}&user_page={}",
            params.advert_page.unwrap_or(1),
            params.user_page.unwrap_or(1)
        ))
        .into_response()
    } else {
        "Failed to proceed mod action".into_response()
    }
}
