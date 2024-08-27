use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Advert {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub published: bool,
}
