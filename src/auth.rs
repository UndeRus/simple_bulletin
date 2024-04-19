use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, bail};
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use data_encoding::HEXUPPER;
use password_auth::verify_password;
use ring::digest::{Context, SHA256};
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool};
use tokio::task;


#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub session_id: Option<String>, 
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        if let Some(session_id) = &self.session_id {
            return session_id.as_bytes();
        }

        &[]
    }
}

#[derive(Clone)]
pub struct AuthBackend {
    pub db: SqlitePool,
}

impl AuthBackend {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    pub password: String,
    pub username: String,
    pub next: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    SQLError(sqlx::Error),
    WrongCreds,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("auth error")
    }
}




#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("select id, username, password_hash, session_id from users where username = ? ")
        .bind(creds.username)
        .fetch_optional(&self.db)
        .await.map_err(|e|{
            println!("SQL Error {}", e);
            AuthError::SQLError(e)})?;

        task::spawn_blocking(move || {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(&creds.password, &user.password_hash).is_ok()))
        })
        .await.map_err(|_|{
            println!("Wrong creds");
            AuthError::WrongCreds
        })?
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("select * from users where id = ?")
        .bind(user_id)
        .fetch_optional(&self.db)
        .await.map_err(|e|AuthError::SQLError(e))?;
        Ok(user)
    }
}
