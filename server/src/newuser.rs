mod auth_models;
mod db;
mod models;

#[tokio::main]
async fn main() {
    println!("Create new user");
    let database = db::create_db("simple_bulletin.db")
        .await
        .expect("Can't open database");
    db::create_new_user(&database, "user", "123").await.unwrap();
}
