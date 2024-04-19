
use axum::{routing::{get, post}, Router};
use axum_login::{login_required, AuthManagerLayerBuilder};

use sqlx;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::auth::AuthBackend;

mod auth;
mod routes;
mod db;

#[tokio::main]
async fn main() {
    
    let app = router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.await.into_make_service()).await.unwrap();
}

async fn router() -> Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let db = db::create_db("simple_bulletin.db").await.unwrap();   

    let backend = AuthBackend::new(db);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();


        Router::new()
            .route(
                "/protected",
                get(|| async { "Gotta be logged in to see me!" }),
            )
            .route("/mod", get(routes::mod_page))
            .route("/item/new", post(routes::item_new))
            .route("/item/new", get(routes::item_new_form))
            .route("/logout", get(routes::logout))
            .route_layer(login_required!(AuthBackend, login_url = "/login"))
            .route("/login", post(routes::login_with_password))
            .route("/login", get(routes::login_form))
            .route("/register", post(routes::register))
            .route("/register", get(routes::register_form))
            .route("/", get(routes::main_board))
            .route("/item/:id", get(routes::item_page))
            .layer(auth_layer)

}
