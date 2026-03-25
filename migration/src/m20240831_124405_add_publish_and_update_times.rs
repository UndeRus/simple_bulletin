use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240822_184202_add_adverts::Adverts;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Adverts::Table)
                    .add_column_if_not_exists(timestamp(Adverts::PublishedTime).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Adverts::Table)
                    .add_column_if_not_exists(timestamp(Adverts::UpdatedTime).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Adverts::Table)
                    .drop_column(Adverts::PublishedTime)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Adverts::Table)
                    .drop_column(Adverts::UpdatedTime)
                    .to_owned(),
            )
            .await
    }
}
