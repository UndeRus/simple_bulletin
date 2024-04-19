use password_auth::generate_hash;

mod db;

#[tokio::main]
async fn main() {
    println!("Create new admin");

    let mut db = db::create_db("simple_bulletin.db").await.unwrap();

    sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)").bind("admin").bind(generate_hash("123")).execute(&db).await.unwrap();
}