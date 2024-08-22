use std::{collections::HashSet, fmt::Display, sync::Arc};

use async_trait::async_trait;
use axum_login::{AuthnBackend, AuthzBackend, UserId};
use password_auth::verify_password;
use serde::Deserialize;
use sqlx::{FromRow, Pool, Sqlite};
use tokio::{sync::RwLock, task};

use crate::auth_models::User;

#[derive(Clone)]
pub struct AuthBackend {
    pub db: Arc<RwLock<Pool<Sqlite>>>,
}

impl AuthBackend {
    pub fn new(db: Arc<RwLock<Pool<Sqlite>>>) -> Self {
        Self { db }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    pub password: String,
    pub username: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromRow)]
pub struct AuthPermission {
    pub name: String,
}

impl From<&str> for AuthPermission {
    fn from(name: &str) -> Self {
        AuthPermission {
            name: name.to_string(),
        }
    }
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
        let db = self.db.read().await;
        let user: Option<Self::User> = sqlx::query_as("select * from users where username = ? ")
            .bind(creds.username)
            .fetch_optional(&*db)
            .await
            .map_err(|e| {
                println!("SQL Error {}", e);
                AuthError::SQLError(e)
            })?;

        task::spawn_blocking(move || {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| {
                let is_ok = verify_password(&creds.password, &user.password_hash).is_ok();
                is_ok
            }))
        })
        .await
        .map_err(|_| {
            println!("Wrong creds");
            AuthError::WrongCreds
        })?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let db = self.db.read().await;
        let user = sqlx::query_as("select * from users where id = ? AND active = TRUE")
            .bind(user_id)
            .fetch_optional(&*db)
            .await
            .map_err(|e| AuthError::SQLError(e))?;
        Ok(user)
    }
}

#[async_trait]
impl AuthzBackend for AuthBackend {
    type Permission = AuthPermission;

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let db = self.db.read().await;
        let permissions: Vec<Self::Permission> = sqlx::query_as(
            r#"
            select distinct permissions.name
            from users
            join users_groups on users.id = users_groups.user_id
            join groups_permissions on users_groups.group_id = groups_permissions.group_id
            join permissions on groups_permissions.permission_id = permissions.id
            where users.id = ? AND users.active = TRUE
            "#,
        )
        .bind(user.id)
        .fetch_all(&*db)
        .await
        .map_err(|e| {
            dbg!(&e);
            AuthError::SQLError(e)
        })?;

        Ok(permissions.into_iter().collect())
    }
}
