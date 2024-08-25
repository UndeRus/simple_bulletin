use std::time::Duration;

use entities::{prelude, users};
use entities::{adverts, prelude::Adverts};
use migration::sea_orm::Database;
use sea_orm::{ColumnTrait, ModelTrait, Set};
use sea_orm::{
    ConnectOptions, DatabaseConnection, EntityOrSelect, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use sea_orm::ActiveModelTrait;

use crate::models::Advert;

pub async fn get_db(uri: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(uri);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("schema"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await.expect("Failed to connect");
    return db;
}

fn map_advert(advert: adverts::Model) -> Advert {
    Advert {
        id: advert.id as i64,
        title: advert.title,
        content: advert.content,
        published: advert.published,
    }
}

pub async fn get_advert_by_id(
    db: &DatabaseConnection,
    user_id: Option<i64>,
    id: i64,
    is_admin: bool,
) -> Result<(Advert, bool), ()> {
    let advert = Adverts::find()
        .filter(adverts::Column::Id.eq(id))
        .one(db)
        .await
        .map_err(|e| ());
    if let Ok(Some(advert)) = advert {
        let advert_user = advert.find_related(prelude::Users).one(db).await;
        if let Ok(Some(advert_user)) = advert_user {
            if let Some(user_id) = user_id {
                if advert_user.id == user_id as i32 {
                    return Ok((map_advert(advert), true));
                }
            } else {
                if advert.published {
                    return Ok((map_advert(advert), false));
                }
            }
        } else if is_admin {
            return Ok((map_advert(advert), true));
        }
    }
    return Err(());
}

pub async fn toggle_user_active(db: &DatabaseConnection, user_id: i64, active: bool) -> Result<(), ()> {
    let user = prelude::Users::find_by_id(user_id as i32).one(db).await.map_err(|_|())?;
    if let Some(user) = user {
        let mut user: users::ActiveModel = user.into();
        user.active = Set(active);
        user.update(db).await.map_err(|_|())?;
    }
    Ok(())
}

pub async fn toggle_advert_publish(
    db: &DatabaseConnection,
    advert_id: i64,
    published: bool,
) -> Result<(), ()> {
    let advert = prelude::Adverts::find_by_id(advert_id as i32).one(db).await.map_err(|_|())?;
    if let Some(advert) = advert {
        let mut advert: adverts::ActiveModel = advert.into();
        advert.published = Set(published);
        advert.update(db).await.map_err(|_|())?;
    }
    Ok(())
}

/*
#[cfg(test)]
mod tests {
    use super::{get_advert_by_id, get_db};

    #[tokio::test]
    async fn connect_db() {
        env_logger::init();

        let db = get_db("sqlite://../simple_bulletin.db").await;

        let advert = get_advert_by_id(&db, Some(2), 1, false).await;
        assert!(advert.is_ok());
        assert!(advert.unwrap().1 == true);

        let advert = get_advert_by_id(&db, None, 1, false).await;
        assert!(advert.is_ok());
        assert!(advert.unwrap().1 == false);

        let advert = get_advert_by_id(&db, None, 1, true).await;
        assert!(advert.is_ok());
        assert!(advert.unwrap().1 == true);
    }
}
*/
