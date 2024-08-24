pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_users;
mod m20240822_113126_create_permissions;
mod m20240822_184202_add_adverts;
mod m20240824_133428_add_active_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users::Migration),
            Box::new(m20240822_113126_create_permissions::Migration),
            Box::new(m20240822_184202_add_adverts::Migration),
            Box::new(m20240824_133428_add_active_user::Migration),
        ]
    }
}
