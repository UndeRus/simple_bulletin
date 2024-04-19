use crate::{auth_models::User, models::Advert};
use password_auth::generate_hash;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    sqlite::SqliteConnectOptions,
    Pool, Sqlite, SqlitePool,
};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn create_db(db_url: &str) -> Result<Pool<Sqlite>, ()> {
    if !sqlx::Sqlite::database_exists(db_url).await.map_err(|e| {
        println!("Failed to check if database exists {}", e);
        ()
    })? {
        sqlx::Sqlite::create_database(db_url).await.map_err(|e| {
            println!("Failed to create database {}", e);
            ()
        })?;
    }

    // Connect to the database
    let connect_options = SqliteConnectOptions::new().filename(&db_url);
    let db = SqlitePool::connect_with(connect_options)
        .await
        .map_err(|_| ())?;

    // Migrate the database
    MIGRATOR.run(&db).await.map_err(|e| {
        println!("Migration error {}", e);
        ()
    })?;
    Ok(db)
}

pub async fn create_new_user(db: &Pool<Sqlite>, username: &str, password: &str) -> Result<(), ()> {
    sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)")
        .bind(username)
        .bind(generate_hash(password))
        .execute(db)
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
    .execute(db)
    .await
    .map_err(|_| ())?;
    Ok(())
}

pub async fn create_new_advert(
    db: &Pool<Sqlite>,
    user_id: i64,
    title: &str,
    content: &str,
) -> Result<i64, ()> {
    let advert_id = sqlx::query("INSERT INTO adverts(title, content) VALUES(?, ?)")
        .bind(title)
        .bind(content)
        .execute(db)
        .await
        .map_err(|e| {
            println!("Failed to create advert {}", e);
            ()
        })?;
    let new_advert_id = advert_id.last_insert_rowid();
    sqlx::query("INSERT INTO users_adverts(user_id, advert_id) VALUES(?, ?)")
        .bind(user_id)
        .bind(new_advert_id)
        .execute(db)
        .await
        .map_err(|e| {
            println!("Failed to join advert to user {}", e);
            ()
        })?;
    Ok(new_advert_id)
}

pub async fn get_advert_by_id(
    db: &Pool<Sqlite>,
    user_id: Option<i64>,
    id: i64,
    is_admin: bool,
) -> Result<(Advert, bool), ()> {
    let mut is_own = false;
    let result: Option<Advert> = if is_admin {
        is_own = true;
        sqlx::query_as("SELECT * FROM adverts WHERE id = ?")
            .bind(id)
    } else if let Some(user_id) = user_id {
        let advert_user_id: i64 = sqlx::query_scalar("SELECT user_id FROM users_adverts WHERE advert_id = ?").bind(id).fetch_one(db).await.map_err(|e| {
            println!("Failed to get item user {}", e);
            ()})?;

            is_own = advert_user_id == user_id;

        sqlx::query_as("SELECT a.id, a.title, a.content, a.published FROM adverts a JOIN users_adverts ua ON a.id = ua.advert_id WHERE a.id = ? AND (a.published = true OR ua.user_id = ?)")
            .bind(id)
            .bind(user_id)
    } else {
        sqlx::query_as("SELECT * FROM adverts WHERE id = ? AND published = true").bind(id) 
    }
           .fetch_optional(db)
.await
    .map_err(|e| {
        println!("Failed to get item {}", e);
        ()})?;
    result.map(|r| (r, is_own)).ok_or(())
}

pub async fn get_main_page(
    db: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Advert>, i64), ()> {
    let result: Vec<Advert> = sqlx::query_as(
        "SELECT * FROM adverts WHERE published = true ORDER BY ID DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| {
        println!("Failed to get adverts: {}", e);
        ()
    })?;

    let total_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM adverts WHERE published = true")
            .fetch_one(db)
            .await
            .map_err(|_| ())?;
    Ok((result, total_count))
}

pub async fn get_mod_page(
    db: &Pool<Sqlite>,
    adverts_offset: i64,
    adverts_limit: i64,
    users_offset: i64,
    users_limit: i64,
) -> Result<((Vec<Advert>, i64), (Vec<User>, i64)), ()> {
    let advert_result: Vec<Advert> =
        sqlx::query_as("SELECT * FROM adverts ORDER BY ID DESC LIMIT ? OFFSET ?")
            .bind(adverts_limit)
            .bind(adverts_offset)
            .fetch_all(db)
            .await
            .map_err(|_| ())?;

    let adverts_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM adverts")
        .fetch_one(db)
        .await
        .map_err(|_| ())?;

    let users_result: Vec<User> =
        sqlx::query_as("SELECT * FROM users ORDER BY ID DESC LIMIT ? OFFSET ?")
            .bind(users_limit)
            .bind(users_offset)
            .fetch_all(db)
            .await
            .map_err(|_| ())?;

    let users_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(db)
        .await
        .map_err(|e| {
            eprintln!("Failed to get users count: {}", e);
            ()
        })?;

    Ok((
        (advert_result, adverts_total_count),
        (users_result, users_total_count),
    ))
}

pub async fn toggle_advert_publish(
    db: &Pool<Sqlite>,
    advert_id: i64,
    published: bool,
) -> Result<(), ()> {
    sqlx::query("UPDATE adverts SET published = ? WHERE id = ?")
        .bind(published)
        .bind(advert_id)
        .execute(db)
        .await
        .map_err(|e| {
            eprintln!("Failed to update advert publish: {}", e);
            ()
        })?;
    Ok(())
}

pub async fn toggle_user_active(db: &Pool<Sqlite>, user_id: i64, active: bool) -> Result<(), ()> {
    sqlx::query("UPDATE users SET active = ? WHERE id = ?")
        .bind(active)
        .bind(user_id)
        .execute(db)
        .await
        .map_err(|e| {
            eprintln!("Failed to update users active: {}", e);
            ()
        })?;
    Ok(())
}

pub async fn check_advert_belong_to_user(
    db: &Pool<Sqlite>,
    user_id: i64,
    advert_id: i64,
) -> Result<bool, ()> {
    let result: Option<i64> = sqlx::query_scalar(
        "SELECT advert_id from users_adverts WHERE user_id = ? AND advert_id = ?",
    )
    .bind(user_id)
    .bind(advert_id)
    .fetch_optional(db)
    .await
    .map_err(|e| {
        eprintln!("Failed to get user advert belong: {}", e);
        ()
    })?;

    Ok(result.is_some())
}

pub async fn get_user_adverts(
    db: &Pool<Sqlite>,
    user_id: i64,
    offset: i64,
    limit: i64,
) -> Result<(Vec<Advert>, i64), ()> {
    let result: Vec<Advert> = sqlx::query_as("SELECT * FROM adverts a JOIN users_adverts u ON a.id = u.advert_id WHERE u.user_id = ? ORDER BY ID DESC LIMIT ? OFFSET ?")
            .bind(user_id)
            .bind(limit)
            .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| {
        println!("Failed to get user adverts: {}", e);
        ()
    })?;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM adverts a JOIN users_adverts u ON a.id = u.advert_id WHERE u.user_id = ? ")
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|_| ())?;
    Ok((result, total_count))
}
