use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, response::Html};
use serde::Deserialize;

use crate::{auth_models::User, db, models::Advert};

const ADVERTS_LIMIT: i64 = 10;
const USERS_LIMIT: i64 = 10;

#[derive(Deserialize)]
pub struct ModPageParams {
    pub before_id_ad: Option<i64>,
    pub after_id_ad: Option<i64>,
    pub before_id_user: Option<i64>,
    pub after_id_user: Option<i64>,
}

#[derive(Template)]
#[template(path = "moderator.html")]
struct ModeratorPageTemplate {
    adverts: Vec<Advert>,
    first_ad_page: bool,
    last_ad_page: bool,
    users: Vec<User>,
    first_user_page: bool,
    last_user_page: bool,
}

//TODO: rework pagination
pub async fn mod_page(Query(params): Query<ModPageParams>) -> impl IntoResponse {
    let (adverts, users) = if let Ok((adverts, users)) = db::get_mod_page(
        ADVERTS_LIMIT,
        params.before_id_ad,
        params.after_id_ad,
        USERS_LIMIT,
        params.before_id_user,
        params.after_id_user,
    )
    .await
    {
        (adverts, users)
    } else {
        return "Failed to load mod page info".into_response();
    };

    let adverts_len = adverts.len();
    let users_len = users.len();

    let template = ModeratorPageTemplate {
        adverts,
        first_ad_page: params.before_id_ad.is_none(),
        last_ad_page: adverts_len < ADVERTS_LIMIT as usize,
        users,
        first_user_page: params.before_id_user.is_none(),
        last_user_page: users_len < USERS_LIMIT as usize,
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