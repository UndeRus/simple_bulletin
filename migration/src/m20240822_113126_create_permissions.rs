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
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(pk_auto(Groups::Id))
                    .col(string(Groups::Name))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Permissions::Table)
                    .if_not_exists()
                    .col(pk_auto(Permissions::Id))
                    .col(string(Permissions::Name))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UsersGroups::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .table(UsersGroups::Table)
                            .col(UsersGroups::UserId)
                            .col(UsersGroups::GroupId),
                    )
                    .col(integer(UsersGroups::UserId))
                    .col(integer(UsersGroups::GroupId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersGroups::Table, UsersGroups::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersGroups::Table, UsersGroups::GroupId)
                            .to(Groups::Table, Groups::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupsPermissions::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .table(GroupsPermissions::Table)
                            .col(GroupsPermissions::GroupId)
                            .col(GroupsPermissions::PermissionId),
                    )
                    .col(integer(GroupsPermissions::GroupId))
                    .col(integer(GroupsPermissions::PermissionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupsPermissions::Table, GroupsPermissions::GroupId)
                            .to(Groups::Table, Groups::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupsPermissions::Table, GroupsPermissions::PermissionId)
                            .to(Permissions::Table, Permissions::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UsersGroups::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GroupsPermissions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Groups {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum Permissions {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum UsersGroups {
    Table,
    #[sea_orm(iden = "user_id")]
    UserId,
    #[sea_orm(iden = "group_id")]
    GroupId,
}

#[derive(DeriveIden)]
enum GroupsPermissions {
    Table,
    #[sea_orm(iden = "group_id")]
    GroupId,
    #[sea_orm(iden = "permission_id")]
    PermissionId,
}
