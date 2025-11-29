use super::entities::guilds::{self, Entity as GuildsEntity, Model as Guild};
use sea_orm::*;

#[derive(Clone)]
pub struct GuildRepository {
    db: DatabaseConnection,
}

impl GuildRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// ギルドを新規作成または既存のギルドを返す
    pub async fn find_or_create(
        &self,
        guild_id: String,
        guild_name: Option<String>,
        owner_id: Option<String>,
    ) -> Result<Guild, DbErr> {
        // 既存のギルドを検索
        if let Some(guild) = self.find_by_guild_id(&guild_id).await? {
            // 既存のギルドが見つかった場合、is_activeをtrueに更新
            if !guild.is_active {
                return self.update_active_status(&guild_id, true).await;
            }
            return Ok(guild);
        }

        // 新規作成
        self.create(guild_id, guild_name, owner_id).await
    }

    /// ギルドを新規作成
    pub async fn create(
        &self,
        guild_id: String,
        guild_name: Option<String>,
        owner_id: Option<String>,
    ) -> Result<Guild, DbErr> {
        let guild = guilds::ActiveModel {
            guild_id: Set(guild_id),
            guild_name: Set(guild_name),
            owner_id: Set(owner_id),
            ..guilds::ActiveModel::new()
        };

        let result = guild.insert(&self.db).await?;
        Ok(result)
    }

    /// guild_idでギルドを検索
    pub async fn find_by_guild_id(&self, guild_id: &str) -> Result<Option<Guild>, DbErr> {
        GuildsEntity::find()
            .filter(guilds::Column::GuildId.eq(guild_id))
            .one(&self.db)
            .await
    }

    /// アクティブなギルド一覧を取得
    #[allow(dead_code)]
    pub async fn find_all_active(&self) -> Result<Vec<Guild>, DbErr> {
        GuildsEntity::find()
            .filter(guilds::Column::IsActive.eq(true))
            .order_by_desc(guilds::Column::JoinedAt)
            .all(&self.db)
            .await
    }

    /// ギルド情報を更新
    pub async fn update(
        &self,
        guild_id: &str,
        guild_name: Option<String>,
        is_active: Option<bool>,
        settings: Option<serde_json::Value>,
    ) -> Result<Guild, DbErr> {
        use chrono::Utc;

        let guild = self
            .find_by_guild_id(guild_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Guild with id {} not found",
                guild_id
            )))?;

        let mut guild: guilds::ActiveModel = guild.into();

        if let Some(name) = guild_name {
            guild.guild_name = Set(Some(name));
        }
        if let Some(active) = is_active {
            guild.is_active = Set(active);
        }
        if let Some(s) = settings {
            guild.settings = Set(s);
        }

        // updated_atを手動で更新
        guild.updated_at = Set(Utc::now().into());

        let updated = guild.update(&self.db).await?;
        Ok(updated)
    }

    /// ギルドのアクティブ状態を更新
    pub async fn update_active_status(
        &self,
        guild_id: &str,
        is_active: bool,
    ) -> Result<Guild, DbErr> {
        self.update(guild_id, None, Some(is_active), None).await
    }

    /// ギルドを削除（論理削除）
    #[allow(dead_code)]
    pub async fn soft_delete(&self, guild_id: &str) -> Result<Guild, DbErr> {
        self.update_active_status(guild_id, false).await
    }
}
