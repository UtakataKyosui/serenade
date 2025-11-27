use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
pub struct GuildCommand {
    name: String,
    #[serde(
        rename = "type",
        default
    )]
    r#type: ApplicationCommandType,
    description: Option<String>
}

impl GuildCommand {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, r#type: ApplicationCommandType::ChatInput, description }
    }
}

#[derive(Serialize, Deserialize,Clone)]
pub enum ApplicationCommandType {
    ChatInput = 1,
    User = 2,
    Message = 3,
    PrimaryEntryPoint = 4,
}

impl Default for ApplicationCommandType {
    fn default() -> Self {
        ApplicationCommandType::ChatInput
    }
}