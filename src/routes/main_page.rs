use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, http::StatusCode, response::Html};
use serde::Deserialize;

use crate::{db, models::Advert};


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
