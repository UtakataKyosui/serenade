use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildCommand {
    name: String,
    #[serde(rename = "type", default)]
    r#type: ApplicationCommandType,
    description: Option<String>,
}

impl GuildCommand {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            name,
            r#type: ApplicationCommandType::ChatInput,
            description,
        }
    }
}

#[macro_export]
macro_rules! guild_command {
    ($name:expr, $description:expr) => {
        GuildCommand::new($name.to_string(), Some($description.to_string()))
    };
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum ApplicationCommandType {
    #[default]
    ChatInput = 1,
    User = 2,
    Message = 3,
    PrimaryEntryPoint = 4,
}

/// Discord Interaction全般を表す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interaction {
    pub id: String,
    #[serde(rename = "type")]
    pub interaction_type: InteractionType,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub member: Option<GuildMember>,
    pub user: Option<User>,
    pub token: String,
    pub version: i32,
    pub data: Option<InteractionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalSubmit = 5,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractionData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub options: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
}

/// GUILD_CREATEイベントのペイロード
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildCreatePayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub joined_at: Option<String>,
}

/// Interactionレスポンス
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    pub response_type: InteractionResponseType,
    pub data: Option<InteractionResponseData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractionResponseData {
    pub content: Option<String>,
    pub embeds: Option<Vec<serde_json::Value>>,
    pub flags: Option<u32>,
}
