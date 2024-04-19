use crate::{auth_models::User, models::Advert};
use password_auth::generate_hash;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::env;

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
        .map_err(|_| ())?;
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
        .map_err(|e| {
            println!("Failed to create advert {}", e);
            ()
        })?;
    let new_advert_id = advert_id.last_insert_rowid();
    sqlx::query("INSERT INTO users_adverts(user_id, advert_id) VALUES(?, ?)")
        .bind(user_id)
        .bind(new_advert_id)
        .execute(&db)
        .await
        .map_err(|e| {
            println!("Failed to join advert to user {}", e);
            ()
        })?;
    Ok(new_advert_id)
}

pub async fn get_advert_by_id(
    user_id: Option<i64>,
    id: i64,
    published: bool,
) -> Result<Advert, ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let result: Option<Advert> =
        sqlx::query_as("SELECT * FROM adverts WHERE id = ? AND published = ?")
            .bind(id)
            .bind(published)
            .fetch_optional(&db)
            .await
            .map_err(|_| ())?;
    result.ok_or(())
}

//TODO: rework pagination
pub async fn get_main_page(
    limit: i64,
    before_id: Option<i64>,
    after_id: Option<i64>,
) -> Result<Vec<Advert>, ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let result: Vec<Advert> = if let Some(before_id) = before_id {
        sqlx::query_as(
            "SELECT * FROM adverts WHERE id < ? AND published = true ORDER BY ID DESC LIMIT ?",
        )
        .bind(before_id)
        .bind(limit)
    } else if let Some(after_id) = after_id {
        sqlx::query_as(
            "SELECT * FROM adverts WHERE id > ? AND published = true ORDER BY ID DESC LIMIT ?",
        )
        .bind(after_id)
        .bind(limit)
    } else {
        sqlx::query_as("SELECT * FROM adverts WHERE published = true ORDER BY ID DESC LIMIT ?")
            .bind(limit)
    }
    .fetch_all(&db)
    .await
    .map_err(|_| ())?;
    Ok(result)
}

//TODO: rework pagination
pub async fn get_mod_page(
    adverts_limit: i64,
    before_ad_id: Option<i64>,
    after_ad_id: Option<i64>,
    users_limit: i64,
    before_user_id: Option<i64>,
    after_user_id: Option<i64>,
) -> Result<(Vec<Advert>, Vec<User>), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let advert_result: Vec<Advert> = if let Some(before_ad_id) = before_ad_id {
        sqlx::query_as("SELECT * FROM adverts WHERE id < ? ORDER BY ID DESC LIMIT ?")
            .bind(before_ad_id)
            .bind(adverts_limit)
    } else if let Some(after_ad_id) = after_ad_id {
        sqlx::query_as("SELECT * FROM adverts WHERE id > ? ORDER BY ID DESC LIMIT ?")
            .bind(after_ad_id)
            .bind(adverts_limit)
    } else {
        sqlx::query_as("SELECT * FROM adverts ORDER BY ID DESC LIMIT ?").bind(adverts_limit)
    }
    .fetch_all(&db)
    .await
    .map_err(|_| ())?;

    let users_result: Vec<User> = if let Some(before_user_id) = before_user_id {
        sqlx::query_as("SELECT * FROM users WHERE id < ? ORDER BY ID DESC LIMIT ?")
            .bind(before_user_id)
            .bind(users_limit)
    } else if let Some(after_user_id) = after_user_id {
        sqlx::query_as("SELECT * FROM users WHERE id > ? ORDER BY ID DESC LIMIT ?")
            .bind(after_user_id)
            .bind(users_limit)
    } else {
        sqlx::query_as("SELECT * FROM users ORDER BY ID DESC LIMIT ?").bind(users_limit)
    }
    .fetch_all(&db)
    .await
    .map_err(|_| ())?;

    Ok((advert_result, users_result))
}
