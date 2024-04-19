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
        .map_err(|_| ())?
        .run(&db)
        .await
        .map_err(|_| ())?;
    Ok(db)
}

pub async fn create_new_user(username: &str, password: &str) -> Result<(), ()> {
    let db = create_db("simple_bulletin.db").await.map_err(|_| ())?;

    sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)")
        .bind(username)
        .bind(generate_hash(password))
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
    .bind("users")
    .execute(&db)
    .await
    .map_err(|_| ())?;
    Ok(())
}
