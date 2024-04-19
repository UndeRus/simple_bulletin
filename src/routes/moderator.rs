use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::Query,
    response::{Html, Redirect},
    Form,
};
use axum_csrf::CsrfToken;
use serde::Deserialize;

use crate::{auth_models::User, db, models::Advert};

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

pub async fn mod_page(token: CsrfToken, Query(params): Query<ModPageParams>) -> impl IntoResponse {
    let csrf_token = if let Ok(csrf_token) = token.authenticity_token() {
        csrf_token
    } else {
        return "Failed to get csrf token".into_response();
    };

    let advert_page = params.advert_page.unwrap_or(1);
    let user_page = params.user_page.unwrap_or(1);
    let adverts_per_page = ADVERTS_LIMIT;
    let users_per_page = USERS_LIMIT;
    let adverts_offset = (advert_page - 1) * adverts_per_page;
    let users_offset = (user_page - 1) * users_per_page;
    let ((adverts, adverts_total_count), (users, users_total_count)) = if let Ok((adverts, users)) =
        db::get_mod_page(adverts_offset, ADVERTS_LIMIT, users_offset, USERS_LIMIT).await
    {
        (adverts, users)
    } else {
        return "Failed to load mod page info".into_response();
    };

    let total_advert_pages = (adverts_total_count as f64 / adverts_per_page as f64).ceil() as i64;
    let total_user_pages = (users_total_count as f64 / users_per_page as f64).ceil() as i64;

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
    token: CsrfToken,
    Query(params): Query<ModPageParams>,
    Form(form): Form<ModEditForm>,
) -> impl IntoResponse {
    if token.verify(&form.csrf_token).is_err() {
        return "Failed to verify csrf".into_response();
    }

    //TODO: add csrf
    let result = match form.action.as_str() {
        ACTIVATE_USER_ACTION => db::toggle_user_active(form.id, true).await,
        DEACTIVATE_USER_ACTION => db::toggle_user_active(form.id, false).await,
        PUBLISH_ADVERT_ACTION => db::toggle_advert_publish(form.id, true).await,
        UNPUBLISH_ADVERT_ACTION => db::toggle_advert_publish(form.id, false).await,
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
