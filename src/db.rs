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
) -> Result<Advert, ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;
    let result: Option<Advert> = if let Some(user_id) = user_id {
        sqlx::query_as("SELECT a.id, a.title, a.content, a.published FROM adverts a JOIN users_adverts ua ON a.id = ua.advert_id WHERE a.id = ? AND (a.published = true OR ua.user_id = ?)")
            .bind(id)
            .bind(user_id)
    } else {
        sqlx::query_as("SELECT * FROM adverts WHERE id = ? AND published = true")
            .bind(id) 
    }     
           .fetch_optional(&db)
.await
    .map_err(|e| {
        println!("Failed to get item {}", e);
        ()})?  ;

    
    result.ok_or(())
}

pub async fn get_main_page(
    limit: i64,
    offset: i64,
) -> Result<(Vec<Advert>, i64), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let result: Vec<Advert> = 
        sqlx::query_as("SELECT * FROM adverts WHERE published = true ORDER BY ID DESC LIMIT ? OFFSET ?")
            .bind(limit)
            .bind(offset)
    .fetch_all(&db)
    .await
    .map_err(|e| {
        println!("Failed to get adverts: {}", e);
        ()
    })?;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM adverts WHERE published = true")
    .fetch_one(&db)
    .await
    .map_err(|_| ())?;
    Ok((result, total_count))
}

pub async fn get_mod_page(
    adverts_offset: i64,
    adverts_limit: i64,
    users_offset: i64,
    users_limit: i64,
) -> Result<((Vec<Advert>, i64), (Vec<User>, i64)), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    let advert_result: Vec<Advert> = sqlx::query_as("SELECT * FROM adverts ORDER BY ID DESC LIMIT ? OFFSET ?").bind(adverts_limit)
    .bind(adverts_offset)
    .fetch_all(&db)
    .await
    .map_err(|_| ())?;

    let adverts_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM adverts")
    .fetch_one(&db)
    .await
    .map_err(|_| ())?;

    let users_result: Vec<User> = 
        sqlx::query_as("SELECT * FROM users ORDER BY ID DESC LIMIT ? OFFSET ?")
        .bind(users_limit)
        .bind(users_offset)
    .fetch_all(&db)
    .await
    .map_err(|_| ())?;

    let users_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
    .fetch_one(&db)
    .await
    .map_err(|e| {
        eprintln!("Failed to get users count: {}", e);
        ()})?;

    Ok(((advert_result, adverts_total_count), (users_result, users_total_count)))
}


pub async fn toggle_advert_publish(advert_id: i64, published: bool) -> Result<(), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;


    sqlx::query("UPDATE adverts SET published = ? WHERE id = ?")
    .bind(published)
    .bind(advert_id)
    .execute(&db)
    .await
    .map_err(|e| {
        eprintln!("Failed to update advert publish: {}", e);
        ()})?;
    Ok(())
}

pub async fn toggle_user_active(user_id: i64, active: bool) -> Result<(), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;


    sqlx::query("UPDATE users SET active = ? WHERE id = ?")
    .bind(active)
    .bind(user_id)
    .execute(&db)
    .await
    .map_err(|e| {
        eprintln!("Failed to update users active: {}", e);
        ()})?;
    Ok(())
}