mod auth_models;
mod db;
mod models;

#[tokio::main]
async fn main() {
    println!("Create new user");
    db::create_new_user("user", "123").await.unwrap();
}
