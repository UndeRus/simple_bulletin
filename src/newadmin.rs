use clap::Parser;
use password_auth::generate_hash;

mod auth_models;
mod db;
mod models;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let admin_username = cli.username;
    let password =
        rpassword::prompt_password("Admin password: ").expect("You must enter admin password");
    create_new_admin(&admin_username, &password).await;
    println!("Admin user {} created", admin_username);
}

async fn create_new_admin(username: &str, password: &str) {
    let db = db::create_db("simple_bulletin.db").await.unwrap();

    sqlx::query("INSERT INTO users(username, password_hash, active) VALUES(?, ?, ?)")
        .bind(username)
        .bind(generate_hash(password))
        .bind(true)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query(
        r#"INSERT INTO
                 users_groups(user_id, group_id)
                 VALUES(
                    (SELECT id FROM users WHERE username = ?),
                    (SELECT id FROM groups WHERE name = ?)
                )"#,
    )
    .bind(username)
    .bind("admins")
    .execute(&db)
    .await
    .unwrap();
}
