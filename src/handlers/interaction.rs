use axum::{extract::State, http::StatusCode, Json};

use crate::{
    commands::structs::{
        Interaction, InteractionResponse, InteractionResponseData, InteractionResponseType,
        InteractionType,
    },
    AppState,
};

/// Discordからのインタラクションを処理するメインハンドラー
pub async fn handle_interaction(
    State(state): State<AppState>,
    Json(interaction): Json<Interaction>,
) -> (StatusCode, Json<InteractionResponse>) {
    // Pingタイプの場合はPongを返す
    if matches!(interaction.interaction_type, InteractionType::Ping) {
        return (
            StatusCode::OK,
            Json(InteractionResponse {
                response_type: InteractionResponseType::Pong,
                data: None,
            }),
        );
    }

    // guild_idが存在する場合、ギルド情報をDBに保存または更新
    if let Some(guild_id) = &interaction.guild_id {
        // ギルドをDB登録（既存の場合は取得のみ）
        if let Err(e) = state
            .guild_repository
            .find_or_create(guild_id.clone(), None, None)
            .await
        {
            tracing::error!("Failed to register guild: {:?}", e);
        }

        // ギルドにコマンドを登録（まだ登録されていない場合）
        if let Err(e) = register_guild_commands(&state, guild_id).await {
            tracing::error!(
                "Failed to register commands for guild {}: {:?}",
                guild_id,
                e
            );
        }
    }

    // アプリケーションコマンドの処理
    if matches!(
        interaction.interaction_type,
        InteractionType::ApplicationCommand
    ) {
        let response = handle_application_command(interaction).await;
        return response;
    }

    // その他のインタラクションタイプ
    (
        StatusCode::OK,
        Json(InteractionResponse {
            response_type: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("Unknown interaction type".to_string()),
                embeds: None,
                flags: None,
            }),
        }),
    )
}

/// アプリケーションコマンドを処理する
async fn handle_application_command(
    interaction: Interaction,
) -> (StatusCode, Json<InteractionResponse>) {
    let command_name = interaction
        .data
        .as_ref()
        .and_then(|d| d.name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    // TODO(human): コマンドのロジックを実装してください

    let response_content = match command_name {
        "ping" => "Pong!",
        "hello" => "Hello, World!",
        _ => "Unknown command",
    };

    (
        StatusCode::OK,
        Json(InteractionResponse {
            response_type: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(response_content.to_string()),
                embeds: None,
                flags: None,
            }),
        }),
    )
}

/// ギルドにコマンドを登録する
pub async fn register_guild_commands(
    state: &AppState,
    guild_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/applications/{}/guilds/{}/commands",
        crate::constants::DISCORD_API_BASE_URL,
        state.application_id,
        guild_id
    );

    let client = reqwest::Client::new();

    // 各コマンドを登録
    for command in &state.commands {
        let response = client
            .post(&url)
            .header("Authorization", format!("Bot {}", state.bot_token))
            .header("Content-Type", "application/json")
            .json(command)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            tracing::error!("Failed to register command: {}", error_text);
            return Err(format!("Failed to register command: {}", error_text).into());
        }
    }

    tracing::info!("Successfully registered commands for guild: {}", guild_id);
    Ok(())
}
