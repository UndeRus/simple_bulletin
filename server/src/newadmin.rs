use clap::Parser;

mod auth;
mod auth_models;
mod db;
mod db_orm;
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
    let db = db_orm::get_db("sqlite://simple_bulletin.db", false)
        .await
        .expect("Failed to created db");
    db_orm::create_new_admin(&db, username, password)
        .await
        .expect("Failed to create admin");
}
