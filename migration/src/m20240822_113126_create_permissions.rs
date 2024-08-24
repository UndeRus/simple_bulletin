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
            .await?;

        let groups = vec!["users", "admins"];

        let gr_queries: Vec<InsertStatement> = groups
            .iter()
            .map(|&group| {
                Query::insert()
                    .into_table(Groups::Table)
                    .columns([Groups::Name])
                    .values_panic([group.into()])
                    .to_owned()
            })
            .collect();

        for query in gr_queries {
            manager.exec_stmt(query).await?
        }

        let permissions = vec!["admin.read", "admin.write", "user.read", "user.write"];

        let perm_queries: Vec<InsertStatement> = permissions
            .iter()
            .map(|&permission| {
                let query = Query::insert()
                    .into_table(Permissions::Table)
                    .columns([Permissions::Name])
                    .values_panic([permission.into()])
                    .to_owned();
                query
            })
            .collect();

        for query in perm_queries {
            manager.exec_stmt(query).await?;
        }

        let db = manager.get_connection();

        let users_group_id =
            get_id_by_table_and_name(db, Groups::Table, Groups::Id, Groups::Name, "users").await?;

        let admins_group_id =
            get_id_by_table_and_name(db, Groups::Table, Groups::Id, Groups::Name, "admins").await?;

        let user_read_perm_id = get_id_by_table_and_name(
            db,
            Permissions::Table,
            Permissions::Id,
            Permissions::Name,
            "user.read",
        )
        .await?;

        let user_write_perm_id = get_id_by_table_and_name(
            db,
            Permissions::Table,
            Permissions::Id,
            Permissions::Name,
            "user.write",
        )
        .await?;


        let admin_read_perm_id = get_id_by_table_and_name(
            db,
            Permissions::Table,
            Permissions::Id,
            Permissions::Name,
            "admin.read",
        )
        .await?;

        let admin_write_perm_id = get_id_by_table_and_name(
            db,
            Permissions::Table,
            Permissions::Id,
            Permissions::Name,
            "admin.write",
        )
        .await?;

        manager.exec_stmt(
            Query::insert()
            .into_table(GroupsPermissions::Table)
            .columns([GroupsPermissions::GroupId, GroupsPermissions::PermissionId])
            .values_panic([users_group_id.into(), user_read_perm_id.into()])
            .values_panic([users_group_id.into(), user_write_perm_id.into()])
            .values_panic([admins_group_id.into(), admin_read_perm_id.into()])
            .values_panic([admins_group_id.into(), admin_write_perm_id.into()])
            .to_owned()).await?;
        /*
        let users_group_id_q = Query::select()
            .column(Groups::Id)
            .from(Groups::Table)
            .and_where(Expr::col(Groups::Name).eq("users"))
            .to_owned();

        let users_group_id_s = db.get_database_backend().build(&users_group_id_q);
        let users_groups_id = db
            .query_one(users_group_id_s)
            .await?
            .expect("Users group not found");
        let users_groups_id: u64 = users_groups_id.try_get("", "id")?;
        */
        Ok(())
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

async fn get_id_by_table_and_name<'a, ICR, ITR>(
    db: &SchemaManagerConnection<'a>,
    table: ITR,
    id_column: ICR,
    name_column: ICR,
    name: &str,
) -> Result<i64, DbErr>
where
    ICR: IntoColumnRef,
    ITR: IntoTableRef,
{
    let users_group_id_q = Query::select()
        .column(id_column)
        .from(table)
        .and_where(Expr::col(name_column).eq(name))
        .to_owned();

    let users_group_id_s = db.get_database_backend().build(&users_group_id_q);
    let users_groups_id = db
        .query_one(users_group_id_s)
        .await?
        .expect("Users group not found");
    let users_groups_id: i64 = users_groups_id.try_get("", "id")?;
    return Ok(users_groups_id);
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
