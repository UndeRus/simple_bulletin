use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Adverts::Table)
                    .if_not_exists()
                    .col(pk_auto(Adverts::Id))
                    .col(string(Adverts::Title))
                    .col(string(Adverts::Content))
                    .col(boolean(Adverts::Published).default(false))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UsersAdverts::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .table(UsersAdverts::Table)
                            .col(UsersAdverts::AdvertId)
                            .col(UsersAdverts::UserId),
                    )
                    .col(integer(UsersAdverts::AdvertId))
                    .col(integer(UsersAdverts::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersAdverts::Table, UsersAdverts::AdvertId)
                            .to(Adverts::Table, Adverts::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersAdverts::Table, UsersAdverts::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Adverts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UsersAdverts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Adverts {
    Table,
    Id,
    Title,
    Content,
    Published,
}

#[derive(DeriveIden)]
enum UsersAdverts {
    Table,
    #[sea_orm(iden = "user_id")]
    UserId,
    #[sea_orm(iden = "advert_id")]
    AdvertId,
}
