use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, response::Html};
use serde::Deserialize;

use crate::{auth_models::User, db, models::Advert};

const ADVERTS_LIMIT: i64 = 10;
const USERS_LIMIT: i64 = 10;


#[derive(Deserialize)]
pub struct ModPageParams {
    user_page: Option<i64>,
    advert_page: Option<i64>
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<i32>,
    per_page: Option<i32>,
}


#[derive(Template)]
#[template(path = "moderator.html")]
struct ModeratorPageTemplate {
    adverts: Vec<Advert>,
    users: Vec<User>,

    advert_page: i64,
    total_advert_pages: i64,
    user_page: i64,
    total_user_pages: i64,
}

pub async fn mod_page(Query(params): Query<ModPageParams>) -> impl IntoResponse {
    let advert_page = params.advert_page.unwrap_or(1);
    let user_page = params.user_page.unwrap_or(1);
    let adverts_per_page = ADVERTS_LIMIT;
    let users_per_page = USERS_LIMIT;
    let adverts_offset = (advert_page - 1) * adverts_per_page;
    let users_offset = (user_page - 1) * users_per_page;
    let ((adverts, adverts_total_count), (users, users_total_count)) = if let Ok((adverts, users)) = db::get_mod_page(
        adverts_offset,
        ADVERTS_LIMIT,
        users_offset,
        USERS_LIMIT,
    )
    .await
    {
        (adverts, users)
    } else {
        return "Failed to load mod page info".into_response();
    };


    let total_advert_pages = (adverts_total_count as f64 / adverts_per_page as f64).ceil() as i64;
    let total_user_pages = (users_total_count as f64 / users_per_page as f64).ceil() as i64;

    let template = ModeratorPageTemplate {
        adverts,
        users,
        advert_page,
        total_advert_pages,
        user_page,
        total_user_pages,
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response()).into_response()
}


#[derive(Deserialize)]
#[serde(tag = "action", content = "id")]
pub enum ModAction {
    ActivateUser(i64),
    DeactivateUser(i64),
    PublishAdvert(i64),
    UnpublishAdvert(i64),
}

#[derive(Deserialize)]

pub struct ModEditForm {
    #[serde(flatten)]
    action: ModAction
}


pub async fn mod_edit() -> impl IntoResponse {

}