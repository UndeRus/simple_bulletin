use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use axum_login::{login_required, permission_required, AuthManagerLayerBuilder};

use db_orm::get_db;
use env_logger::init;
use sea_orm::DatabaseConnection;
use sqlx::{Pool, Sqlite};
use tokio::sync::RwLock;
use tower_sessions::{MemoryStore, SessionManagerLayer};

use crate::auth::AuthBackend;

mod auth;
mod auth_models;
mod db;
mod db_orm;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    //env_logger::init();
    tracing_subscriber::fmt().init();


    let app = router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.await.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    db: Arc<RwLock<Pool<Sqlite>>>,

    db1: Arc<RwLock<DatabaseConnection>>,
}

async fn router() -> Router {
    let csrf_config = CsrfConfig::default();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let db = db::create_db("simple_bulletin.db")
        .await
        .expect("Failed to create db");
    let db1 = get_db("sqlite://simple_bulletin.db").await;
    let db1 = Arc::new(RwLock::new(db1));

    let db = Arc::new(RwLock::new(db.clone()));

    let state = AppState { 
        db: db.clone(),
        db1: db1.clone(),
     };

    let backend = AuthBackend::new(db, db1);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    Router::new()
        .merge(mod_router())
        .merge(auth_router())
        .merge(user_router())
        .route("/register", post(routes::register))
        .route("/register", get(routes::register_form))
        .route("/", get(routes::main_board))
        .route("/item/:id", get(routes::item_page))
        .route("/item/:id", post(routes::item_page_edit))
        .layer(auth_layer)
        .layer(CsrfLayer::new(csrf_config))
        .with_state(state)
}

fn mod_router() -> Router<AppState> {
    Router::new()
        .route("/mod", get(routes::mod_page))
        .route_layer(permission_required!(
            AuthBackend,
            login_url = "/login",
            "admin.read"
        ))
        .route("/mod", post(routes::mod_edit))
}

fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/logout", get(routes::logout))
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
        .route("/login", post(routes::login_with_password))
        .route("/login", get(routes::login_form))
}

fn user_router() -> Router<AppState> {
    Router::new()
        .route("/item/new", post(routes::item_new))
        .route_layer(permission_required!(
            AuthBackend,
            login_url = "/login",
            "user.read"
        ))
        .route("/item/new", get(routes::item_new_form))
        .route_layer(permission_required!(
            AuthBackend,
            login_url = "/login",
            "user.write"
        ))
        .route("/profile", get(routes::profile))
        .route_layer(permission_required!(
            AuthBackend,
            login_url = "/login",
            "user.read"
        ))
}
