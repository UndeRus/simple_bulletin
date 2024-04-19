use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, response::Html};
use serde::Deserialize;

use crate::{db, models::Advert};

const MAIN_PAGE_LIMIT: i64 = 10;

#[derive(Template)]
#[template(path = "main.html")]
pub struct MainPageTemplate {
    adverts: Vec<Advert>,
    first_page: bool,
    last_page: bool,
}

#[derive(Deserialize)]
pub struct MainPageParams {
    pub before_id: Option<i64>,
    pub after_id: Option<i64>,
}

//TODO: rework pagination
pub async fn main_board(Query(params): Query<MainPageParams>) -> impl IntoResponse {
    let adverts = if let Ok(adverts) = db::get_main_page(MAIN_PAGE_LIMIT, params.before_id, params.after_id).await {
        adverts
    } else {
        return "Main page error".into_response();
    };

    let adverts_len = adverts.len();
    let template = MainPageTemplate {
        adverts,
        first_page: params.before_id.is_none(),
        last_page: adverts_len < MAIN_PAGE_LIMIT as usize,
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response()).into_response()
}
