use std::env;

use axum::{
    http::StatusCode, middleware, response::IntoResponse, routing::{post}, Json, Router
};
use serde::{Serialize, 
    Deserialize};
use ed25519_dalek::SigningKey;
use tower::ServiceBuilder;
use hex::FromHex;

mod middlewares;
mod interactions;

use crate::middlewares::{verify_signature,logging_middleware};

#[derive(Clone)]
struct AppState {
    pub_key: SigningKey,
    // タイムスタンプ許容範囲（秒）: 300 = 5分
    allowed_clock_skew_secs: i64,
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
    };

    // build our application with a single route
    let app = Router::new()
        .route("/",post(pong))
        .layer(ServiceBuilder::new().layer(
            middleware::from_fn_with_state(state, verify_signature)
        ))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(logging_middleware))
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn pong() -> impl IntoResponse{
    (StatusCode::OK,Json(PongResponse{r#type:1}))
}