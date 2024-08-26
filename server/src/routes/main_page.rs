use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::AuthBackend, db_orm, models::Advert, AppState};

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
    State(state): State<AppState>,
    Query(params): Query<MainPageParams>,
    auth_session: AuthSession<AuthBackend>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = MAIN_PAGE_LIMIT;
    let db = state.db1.read().await;
    let (adverts, total_pages) =
        if let Ok(adverts) = db_orm::get_main_page(&db, per_page, page - 1).await {
            adverts
        } else {
            return "Main page error".into_response();
        };

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
