use std::collections::HashSet;
use std::time::Duration;

use entities::{adverts, prelude::Adverts};
use entities::{
    groups, groups_permissions, permissions, prelude, users, users_adverts, users_groups,
};
use migration::sea_orm::Database;
use password_auth::{generate_hash, verify_password};
use sea_orm::{ActiveModelTrait, QuerySelect, TransactionTrait};
use sea_orm::{ColumnTrait, ModelTrait, Set};
use sea_orm::{
    ConnectOptions, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    RelationTrait,
};
use tokio::task;
use sea_orm_migration::prelude::*;

use crate::auth::AuthPermission;
use crate::auth_models::User;
use crate::models::Advert;

pub async fn get_db(uri: &str) -> Result<DatabaseConnection, ()> {
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

    let db = Database::connect(opt).await.map_err(|e| {
        println!("Failed to create database {}", e);
        ()
    })?;
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to migrate");
    return Ok(db);
}

fn map_advert(advert: &adverts::Model) -> Advert {
    Advert {
        id: advert.id as i64,
        title: advert.title.clone(),
        content: advert.content.clone(),
        published: advert.published,
    }
}

fn map_user(user: &users::Model) -> User {
    User {
        id: user.id as i64,
        username: user.username.clone(),
        password_hash: user.password_hash.clone(),
        active: user.active,
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
        .map_err(|_| ());
    if let Ok(Some(advert)) = advert {
        let advert_user = advert.find_related(prelude::Users).one(db).await;
        if let Ok(Some(advert_user)) = advert_user {
            if let Some(user_id) = user_id {
                if advert_user.id == user_id as i32 {
                    return Ok((map_advert(&advert), true));
                }
            } else {
                if advert.published {
                    return Ok((map_advert(&advert), false));
                }
            }
        } else if is_admin {
            return Ok((map_advert(&advert), true));
        }
    }
    return Err(());
}

pub async fn toggle_user_active(
    db: &DatabaseConnection,
    user_id: i64,
    active: bool,
) -> Result<(), ()> {
    let user = prelude::Users::find_by_id(user_id as i32)
        .one(db)
        .await
        .map_err(|_| ())?;
    if let Some(user) = user {
        let mut user: users::ActiveModel = user.into();
        user.active = Set(active);
        user.update(db).await.map_err(|_| ())?;
    }
    Ok(())
}

pub async fn toggle_advert_publish(
    db: &DatabaseConnection,
    advert_id: i64,
    published: bool,
) -> Result<(), ()> {
    let advert = prelude::Adverts::find_by_id(advert_id as i32)
        .one(db)
        .await
        .map_err(|_| ())?;
    if let Some(advert) = advert {
        let mut advert: adverts::ActiveModel = advert.into();
        advert.published = Set(published);
        advert.update(db).await.map_err(|_| ())?;
    }
    Ok(())
}

pub async fn get_mod_page(
    db: &DatabaseConnection,
    adverts_page: u64,
    adverts_limit: i64,
    users_page: u64,
    users_limit: i64,
) -> Result<((Vec<Advert>, i64), (Vec<User>, i64)), ()> {
    let adverts_paginator = prelude::Adverts::find().paginate(db, adverts_limit as u64);

    let adverts_total_pages = adverts_paginator.num_pages().await.map_err(|_| ())?;
    let adverts_result: Vec<Advert> = adverts_paginator
        .fetch_page(adverts_page)
        .await
        .map_err(|_| ())?
        .iter()
        .map(|a| map_advert(a))
        .collect();

    let users_paginator = prelude::Users::find().paginate(db, users_limit as u64);

    let users_total_pages = users_paginator.num_pages().await.map_err(|_| ())?;
    let users_result: Vec<User> = users_paginator
        .fetch_page(users_page)
        .await
        .map_err(|_| ())?
        .iter()
        .map(|u| map_user(u))
        .collect();

    Ok((
        (adverts_result, adverts_total_pages as i64),
        (users_result, users_total_pages as i64),
    ))
}

pub async fn check_advert_belong_to_user(
    db: &DatabaseConnection,
    user_id: i64,
    advert_id: i64,
) -> Result<bool, ()> {
    let advert = prelude::Adverts::find_by_id(advert_id as i32)
        .one(db)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    let advert_user = advert
        .find_related(prelude::Users)
        .one(db)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    Ok(advert_user.id == user_id as i32)
}

pub async fn get_main_page(
    db: &DatabaseConnection,
    limit: i64,
    page: i64,
) -> Result<(Vec<Advert>, i64), ()> {
    let paginator = prelude::Adverts::find()
        .filter(adverts::Column::Published.eq(true))
        .order_by_desc(adverts::Column::Id)
        .paginate(db, limit as u64);
    let pages = paginator.num_pages().await.map_err(|_| ())? as i64;
    let adverts: Vec<Advert> = paginator
        .fetch_page(page as u64)
        .await
        .map_err(|_| ())?
        .iter()
        .map(|a| map_advert(a))
        .collect();
    Ok((adverts, pages))
}

pub async fn create_new_advert(
    db: &DatabaseConnection,
    user_id: i64,
    title: &str,
    content: &str,
) -> Result<i64, ()> {
    let new_advert = adverts::ActiveModel {
        title: Set(title.to_owned()),
        content: Set(content.to_owned()),
        ..Default::default()
    };

    let txn = db.begin().await.map_err(|e| {
        println!("Failed to create advert start transaction {}", e);
        ()
    })?;

    let advert = new_advert.insert(db).await.map_err(|e| {
        println!("Failed to create advert {}", e);
        ()
    })?;

    let advert_id = advert.id;

    let new_advert_user = users_adverts::ActiveModel {
        advert_id: Set(advert_id),
        user_id: Set(user_id as i32),
    };

    new_advert_user.insert(db).await.map_err(|e| {
        println!("Failed to create advert create relation {}", e);
        ()
    })?;

    txn.commit().await.map_err(|e| {
        println!("Failed to create advert finish transaction {}", e);
        ()
    })?;

    Ok(advert_id.into())
}

pub async fn get_user_adverts(
    db: &DatabaseConnection,
    user_id: i64,
    page: i64,
    limit: i64,
) -> Result<(Vec<Advert>, i64), ()> {
    let adverts_paginator = prelude::Adverts::find()
        .join(
            sea_orm::JoinType::Join,
            adverts::Relation::UsersAdverts.def(),
        )
        .filter(users_adverts::Column::UserId.eq(user_id))
        .order_by_desc(adverts::Column::Id)
        .paginate(db, limit as u64);
    let result: Vec<Advert> = adverts_paginator
        .fetch_page(page as u64)
        .await
        .map_err(|_| ())?
        .iter()
        .map(|a| map_advert(a))
        .collect();
    let pages = adverts_paginator.num_pages().await.map_err(|_| ())?;
    Ok((result, pages as i64))
}

pub async fn create_new_user(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<(), ()> {
    let txn = db.begin().await.map_err(|_| ())?;
    let new_user = users::ActiveModel {
        username: Set(username.to_owned()),
        password_hash: Set(generate_hash(password)),
        active: Set(false),
        ..Default::default()
    };

    let new_user_model = new_user.insert(&txn).await.map_err(|_| ())?;

    let user_group = prelude::Groups::find()
        .filter(groups::Column::Name.eq("users"))
        .one(db)
        .await
        .map_err(|_| ())?
        .ok_or(())?;

    let new_user_id = new_user_model.id.clone();
    let new_user_group = users_groups::ActiveModel {
        user_id: Set(new_user_id),
        group_id: Set(user_group.id),
    };
    new_user_group.insert(&txn).await.map_err(|_| ())?;
    txn.commit().await.map_err(|_| ())?;
    Ok(())
}

pub async fn check_user<'a>(
    db: &DatabaseConnection,
    username: &str,
    password: &'a str,
) -> Result<Option<User>, ()> {
    let user = prelude::Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|_| ())?;

    let password = password.to_owned();

    task::spawn_blocking(move || {
        Ok(user
            .filter(|u| {
                let is_ok = verify_password(password, &u.password_hash).is_ok();
                is_ok
            })
            .map(|u| map_user(&u)))
    })
    .await
    .map_err(|_| ())?
}

pub async fn get_active_user_by_id(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Option<User>, ()> {
    let result = prelude::Users::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|_| ())?;

    Ok(result.map(|u| map_user(&u)))
}

pub async fn get_active_user_permissions(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<HashSet<AuthPermission>, ()> {
    let permissions: HashSet<AuthPermission> = prelude::Permissions::find()
        .join(
            sea_orm::JoinType::Join,
            permissions::Relation::GroupsPermissions.def(),
        )
        .join(
            sea_orm::JoinType::Join,
            groups_permissions::Relation::Groups.def(),
        )
        .join(sea_orm::JoinType::Join, groups::Relation::UsersGroups.def())
        .join(sea_orm::JoinType::Join, users_groups::Relation::Users.def())
        .filter(users_groups::Column::UserId.eq(user_id))
        .filter(users::Column::Active.eq(true))
        .all(db)
        .await
        .map_err(|e| {
            println!("Failed to fetch permissions {}", e);
            ()
        })?
        .iter()
        .map(|m| AuthPermission::from(m.name.as_ref()))
        .collect();
    Ok(permissions)
}

pub async fn create_new_admin(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<(), ()> {
    let txn = db.begin().await.map_err(|_| ())?;
    let new_user = users::ActiveModel {
        username: Set(username.to_owned()),
        password_hash: Set(generate_hash(password)),
        active: Set(true),
        ..Default::default()
    };

    let new_user_model = new_user.insert(&txn).await.map_err(|_| ())?;

    let admin_group = prelude::Groups::find()
        .filter(groups::Column::Name.eq("admins"))
        .one(db)
        .await
        .map_err(|_| ())?
        .ok_or(())?;

    let new_user_id = new_user_model.id.clone();
    let new_user_group = users_groups::ActiveModel {
        user_id: Set(new_user_id),
        group_id: Set(admin_group.id),
    };
    new_user_group.insert(&txn).await.map_err(|_| ())?;
    txn.commit().await.map_err(|_| ())?;
    Ok(())
}
