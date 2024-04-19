use askama_axum::IntoResponse;

pub async fn mod_page() -> impl IntoResponse {
    "Mod page"
}
