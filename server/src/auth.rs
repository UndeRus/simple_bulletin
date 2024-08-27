use std::{collections::HashSet, fmt::Display, sync::Arc};

use async_trait::async_trait;
use axum_login::{AuthnBackend, AuthzBackend, UserId};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{auth_models::User, db_orm};

#[derive(Clone)]
pub struct AuthBackend {
    pub db1: Arc<RwLock<DatabaseConnection>>,
}

impl AuthBackend {
    pub fn new(db1: Arc<RwLock<DatabaseConnection>>) -> Self {
        Self { db1 }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    pub password: String,
    pub username: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    ORMError,
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
        let db = self.db1.read().await;

        db_orm::check_user(&db, &creds.username, &creds.password)
            .await
            .map_err(|_| {
                println!("Wrong creds");
                AuthError::WrongCreds
            })
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let db = self.db1.read().await;
        let user = db_orm::get_active_user_by_id(&db, *user_id as i32)
            .await
            .map_err(|_| AuthError::ORMError)?;
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
        let db = self.db1.read().await;

        db_orm::get_active_user_permissions(&db, user.id as i32)
            .await
            .map_err(|_| AuthError::ORMError)
    }
}
