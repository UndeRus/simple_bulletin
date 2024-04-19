use password_auth::generate_hash;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::env;
use crate::models::Advert;

pub async fn create_db(db_url: &str) -> Result<Pool<Sqlite>, ()> {
    use std::path::Path;
    if !sqlx::Sqlite::database_exists(db_url)
        .await
        .map_err(|_| ())?
    {
        sqlx::Sqlite::create_database(db_url)
            .await
            .map_err(|_| ())?;
    }

    // Connect to the database
    let db = SqlitePool::connect(&db_url).await.map_err(|_| ())?;

    // Migrate the database
    let migrations = if env::var("RUST_ENV") == Ok("production".to_string()) {
        // Productions migrations dir
        std::env::current_exe()
            .map_err(|_| ())?
            .join("./migrations")
    } else {
        // Development migrations dir
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| ())?;
        Path::new(&crate_dir).join("./migrations")
    };
    sqlx::migrate::Migrator::new(migrations)
        .await
        .map_err(|e| {
            println!("Migration error {}", e);
            ()
        })?
        .run(&db)
        .await
        .map_err(|e| {
            println!("Migration error {}", e);
            ()
        })?;
    Ok(db)
}

pub async fn create_new_user(username: &str, password: &str) -> Result<(), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)")
        .bind(username)
        .bind(generate_hash(password))
        .execute(&db)
        .await
        .map_err(|_|())?;
    sqlx::query(
        r#"INSERT INTO
                 users_groups(user_id, group_id)
                 VALUES(
                    (SELECT id FROM users WHERE username = ?),
                    (SELECT id FROM groups WHERE name = ?)
                )"#,
    )
    .bind(username)
    .bind("users")
    .execute(&db)
    .await
    .map_err(|_| ())?;
    Ok(())
}

pub async fn create_new_advert(user_id: i64, title: &str, content: &str) -> Result<i64, ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;
    let advert_id = sqlx::query("INSERT INTO adverts(title, content) VALUES(?, ?)")
    .bind(title)
    .bind(content)
    .execute(&db)
    .await
    .map_err(|e|{
        println!("Failed to create advert {}", e);
        ()})?;
        let new_advert_id = advert_id.last_insert_rowid();
    sqlx::query("INSERT INTO users_adverts(user_id, advert_id) VALUES(?, ?)")
    .bind(user_id)
    .bind(new_advert_id)
    .execute(&db)
    .await
    .map_err(|e|{
        println!("Failed to join advert to user {}", e);
        ()
    })?;
    Ok(new_advert_id)
}

pub async fn get_advert_by_id(id: i64) -> Result<Advert, ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let result: Option<Advert> = sqlx::query_as("SELECT * FROM adverts WHERE id = ?").bind(id).fetch_optional(& db).await.map_err(|_|())?;
    result.ok_or(())
}