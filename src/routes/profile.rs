use askama_axum::IntoResponse;
use axum_login::AuthSession;


use crate::auth::AuthBackend;

pub async fn profile(auth_session: AuthSession<AuthBackend>) -> impl IntoResponse {
    auth_session.user.unwrap().username
}