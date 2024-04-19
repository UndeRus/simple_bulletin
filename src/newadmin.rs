use password_auth::generate_hash;

mod db;
mod models;

#[tokio::main]
async fn main() {
    println!("Create new admin");
    create_new_admin("admin", "123").await;
}

async fn create_new_admin(username: &str, password: &str) {
    let db = db::create_db("simple_bulletin.db").await.unwrap();

    sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)").bind(username).bind(generate_hash(password)).execute(&db).await.unwrap();
    sqlx::query(r#"INSERT INTO
                 users_groups(user_id, group_id)
                 VALUES(
                    (SELECT id FROM users WHERE username = ?),
                    (SELECT id FROM groups WHERE name = ?)
                )"#).bind(username).bind("admins").execute(&db).await.unwrap();
}