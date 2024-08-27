mod auth;
mod auth_models;
mod db_orm;
mod models;

#[tokio::main]
async fn main() {
    println!("Create new user");
    let db = db_orm::get_db("simple_bulletin.db")
        .await
        .expect("Can't open database");
    db_orm::create_new_user(&db, "user", "123")
        .await
        .expect("Failed to create user");
}
