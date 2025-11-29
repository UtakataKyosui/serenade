use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Guilds::Table)
                    .if_not_exists()
                    .col(pk_auto(Guilds::Id))
                    .col(string_uniq(Guilds::GuildId))
                    .col(string_null(Guilds::GuildName))
                    .col(string_null(Guilds::OwnerId))
                    .col(timestamp_with_time_zone(Guilds::JoinedAt))
                    .col(boolean(Guilds::IsActive).default(true))
                    .col(json(Guilds::Settings).default("{}"))
                    .col(timestamp_with_time_zone(Guilds::CreatedAt))
                    .col(timestamp_with_time_zone(Guilds::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // インデックスの作成
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_guilds_guild_id")
                    .table(Guilds::Table)
                    .col(Guilds::GuildId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_guilds_is_active")
                    .table(Guilds::Table)
                    .col(Guilds::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_guilds_joined_at")
                    .table(Guilds::Table)
                    .col(Guilds::JoinedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guilds::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Guilds {
    Table,
    Id,
    GuildId,
    GuildName,
    OwnerId,
    JoinedAt,
    IsActive,
    Settings,
    CreatedAt,
    UpdatedAt,
}
