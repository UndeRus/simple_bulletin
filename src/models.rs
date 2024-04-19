use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, Clone, FromRow)]
pub struct Advert {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub published: bool,
}