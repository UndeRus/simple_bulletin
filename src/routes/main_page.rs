use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, response::Html};
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::AuthBackend, db, models::Advert};

const MAIN_PAGE_LIMIT: i64 = 10;

#[derive(Template)]
#[template(path = "main.html")]
pub struct MainPageTemplate {
    adverts: Vec<Advert>,
    total_pages: i64,
    page: i64,
    logged_in: bool,
}

#[derive(Deserialize)]
pub struct MainPageParams {
    page: Option<i64>,
}

pub async fn main_board(
    Query(params): Query<MainPageParams>,
    auth_session: AuthSession<AuthBackend>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = MAIN_PAGE_LIMIT;
    let offset = (page - 1) * per_page;

    let (adverts, total_count) =
        if let Ok(adverts) = db::get_main_page(MAIN_PAGE_LIMIT, offset).await {
            adverts
        } else {
            return "Main page error".into_response();
        };
    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;


    let logged_in = auth_session.user.is_some();
    let template = MainPageTemplate {
        adverts,
        total_pages,
        page,
        logged_in,
    };
    let reply_html = template.render().unwrap();
    (StatusCode::OK, Html(reply_html).into_response()).into_response()
}
