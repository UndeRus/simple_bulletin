use clap::Parser;

mod auth;
mod auth_models;
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

    let username = cli.username;
    let password =
        rpassword::prompt_password("User password: ").expect("You must enter user password");
    println!("Create new user");
    let db = db_orm::get_db("sqlite://simple_bulletin.db", false)
        .await
        .expect("Can't open database");
    db_orm::create_new_user(&db, &username, &password)
        .await
        .expect("Failed to create user");
}
