use std::vec;

use axum::{middleware, routing::post, Router};
use ed25519_dalek::SigningKey;
use hex::FromHex;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::SecretStore;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod commands;
mod constants;
mod db;
mod handlers;
mod middlewares;

use crate::{
    commands::structs::GuildCommand, db::GuildRepository, handlers::handle_interaction,
    middlewares::verify_signature,
};

#[derive(Clone)]
struct AppState {
    pub_key: SigningKey,
    // タイムスタンプ許容範囲（秒）: 300 = 5分
    allowed_clock_skew_secs: i64,
    commands: Vec<GuildCommand>,
    guild_repository: GuildRepository,
    application_id: String,
    bot_token: String,
}

// #[tokio::main]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] db_url: String,
) -> ShuttleAxum {
    // データベース接続の作成
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // マイグレーション実行
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    // 公開鍵の取得
    let pk_hex = secrets
        .get("DISCORD_PUBLIC_KEY")
        .expect("DISCORD_PUBLIC_KEY must be set (hex-encoded ed25519 public key)");
    let pk_bytes =
        <[u8; 32]>::from_hex(pk_hex.as_str()).expect("DISCORD_PUBLIC_KEY must be 32 bytes hex");
    let pub_key = SigningKey::from(pk_bytes);

    // アプリケーションIDの取得
    let application_id = secrets
        .get("DISCORD_APPLICATION_ID")
        .expect("DISCORD_APPLICATION_ID must be set");

    // Bot Tokenの取得
    let bot_token = secrets
        .get("DISCORD_BOT_TOKEN")
        .expect("DISCORD_BOT_TOKEN must be set");

    // リポジトリの作成
    let guild_repository = GuildRepository::new(db.clone());

    let state = AppState {
        pub_key,
        allowed_clock_skew_secs: 300, // 5分
        commands: vec![
            guild_command!("ping", "Replies with Pong!"),
            guild_command!("hello", "Replies with Hello, World!"),
        ],
        guild_repository,
        application_id,
        bot_token,
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", post(handle_interaction))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    verify_signature,
                ))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state);

    Ok(app.into())
}
