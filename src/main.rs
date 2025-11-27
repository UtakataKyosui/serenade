use std::{env, vec};

use axum::{
    http::StatusCode, middleware, response::IntoResponse, routing::{post}, Json, Router
};
use serde::{Serialize, 
    Deserialize};
use ed25519_dalek::SigningKey;
use tower_http::{trace::TraceLayer};
use tower::{Layer, ServiceBuilder};
use hex::FromHex;

mod middlewares;
mod commands;
mod constants;

use crate::{commands::structs::GuildCommand, middlewares::{guild_initialize_command, verify_signature}};

#[derive(Clone)]
struct AppState {
    pub_key: SigningKey,
    // タイムスタンプ許容範囲（秒）: 300 = 5分
    allowed_clock_skew_secs: i64,
    commands: Vec<GuildCommand>,
}

#[derive(Serialize,Deserialize)]
struct PongResponse {
    r#type: i8
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    dotenvy::from_filename(".env.local").ok();

    // 環境変数から公開鍵（hex）を読み取る
    let pk_hex = env::var("DISCORD_PUBLIC_KEY")
        .expect("DISCORD_PUBLIC_KEY must be set (hex-encoded ed25519 public key)");
    let pk_bytes = <[u8; 32]>::from_hex(pk_hex.as_str())
        .expect("DISCORD_PUBLIC_KEY must be 32 bytes hex");
    let pub_key = SigningKey::from(pk_bytes);

    let state = AppState {
        pub_key,
        allowed_clock_skew_secs: 300, // 5分
        commands: vec![
            GuildCommand::new("ping".to_string(), Some("Replies with Pong!".to_string())),
            GuildCommand::new("hello".to_string(), Some("Replies with Hello, World!".to_string())),
        ],
    };

    // build our application with a single route
    let app = Router::new()
        .route("/",post(pong))
        .layer(
            ServiceBuilder::new().layer(
                middleware::from_fn_with_state(state.clone(), verify_signature)
            )
            .layer(TraceLayer::new_for_http())
            .layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(state.clone(), guild_initialize_command))
        ));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn pong() -> impl IntoResponse{
    (StatusCode::OK,Json(PongResponse{r#type:1}))
}