use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::{Query, State}, response::Html};
use axum_csrf::CsrfToken;
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{auth::AuthBackend, db, models::Advert, AppState};

const PROFILE_PAGE_LIMIT: i64 = 10;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfilePageTemplate {
    adverts: Vec<Advert>,
    total_pages: i64,
    page: i64,
    logged_in: bool,
}

#[derive(Deserialize)]
pub struct ProfilePageParams {
    page: Option<i64>,
}

pub async fn profile(
    State(state): State<AppState>, 
    token: CsrfToken,
    auth_session: AuthSession<AuthBackend>,
    Query(path): Query<ProfilePageParams>,
) -> impl IntoResponse {
    let user = if let Some(user) = auth_session.user {
        user
    } else {
        return "User not found".into_response();
    };

    let page = path.page.unwrap_or(1);
    let per_page = PROFILE_PAGE_LIMIT;
    let offset = (page - 1) * per_page;

    let db= state.db.read().await;
    let (adverts, total_count) =
        if let Ok((adverts, total_count)) = db::get_user_adverts(&db, user.id, offset, per_page).await {
            (adverts, total_count)
        } else {
            return "Failed to load profile".into_response();
        };

    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;

    let template = ProfilePageTemplate {
        adverts,
        total_pages,
        page,
        logged_in: true,
    };
    let reply_html = template.render().unwrap();
    (token, Html(reply_html).into_response()).into_response()
}
