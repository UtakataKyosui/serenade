use std::{convert::Infallible, time::{SystemTime, UNIX_EPOCH}};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response}
};
use hex::FromHex;
use ed25519_dalek::Signature;
use hyper::{Request};


pub async fn verify_signature(
    State(state): State<super::AppState>,
    req: Request<Body>,
    next: Next
) -> Result<impl IntoResponse, Infallible> {
    // 必要ヘッダ取得
    let sig_hex = match req.headers().get("X-Signature-Ed25519") {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return Ok((StatusCode::UNAUTHORIZED, "invalid signature header").into_response()),
        },
        None => return Ok((StatusCode::UNAUTHORIZED, "missing signature header").into_response()),
    };

    let timestamp = match req.headers().get("X-Signature-Timestamp") {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return Ok((StatusCode::UNAUTHORIZED, "invalid timestamp header").into_response()),
        },
        None => return Ok((StatusCode::UNAUTHORIZED, "missing timestamp header").into_response()),
    };

    let (parts, body) = req.into_parts();

    // 本文を全部読む（後で再利用できるようにバッファに保存して Request を再構築する）
    let whole_body = match to_bytes(body,usize::MAX).await {
        Ok(b) => b,
        Err(_) => return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to read body").into_response()),
    };

    // タイムスタンプのリプレイ防止チェック（秒）
    if let Ok(ts_i64) = timestamp.parse::<i64>() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let diff = (now - ts_i64).abs();
        if diff > state.allowed_clock_skew_secs {
            return Ok((StatusCode::UNAUTHORIZED, "timestamp out of allowed range").into_response());
        }
    } else {
        return Ok((StatusCode::UNAUTHORIZED, "invalid timestamp format").into_response());
    }

    // 検証対象メッセージは timestamp + body (discord の仕様)
    let mut message = Vec::with_capacity(timestamp.len() + whole_body.len());
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(&whole_body[..]);

    // signature を hex -> bytes に変換
    let sig_byte = match Vec::from_hex(sig_hex) {
        Ok(b) => b,
        Err(_) => return Ok((StatusCode::UNAUTHORIZED, "invalid signature hex").into_response()),
    };

    // signature 長チェック
    if sig_byte.len() != 64 {
        return Ok((StatusCode::UNAUTHORIZED, "invalid signature length").into_response());
    }

    let sig_bytes: [u8; 64] = sig_byte.try_into().unwrap();
    
    let signature = Signature::from_bytes(&sig_bytes);

    // 検証実行
    if let Err(_) = state.pub_key.verify(&message, &signature) {
        return Ok((StatusCode::UNAUTHORIZED, "signature verification failed").into_response());
    }

    // 検証成功 -> 元のリクエストを再構築して downstream に渡す
    // 新しい Body を作る（Bytes -> Body）
    let new_body = Body::from(whole_body.clone());
    let req = Request::from_parts(parts, new_body);

    // Next に渡す（これが handler に到達する）
    Ok(next.run(req).await.into_response())
}

pub async fn guild_initialize_command(
    State(state): State<super::AppState>,
    req: Request<Body>,
    next: Next
) -> Response{
    let commands = state.commands.clone();
    let application_id = std::env::var("DISCORD_APPLICATION_ID")
        .expect("DISCORD_APPLICATION_ID must be set");
    let guild_id = std::env::var("DISCORD_GUILD_ID")
        .expect("DISCORD_GUILD_ID must be set");

    let _  = commands.iter().map(async |command| {
        let url = format!("{}/applications/{}/guilds/{}/commands",
            crate::constants::DISCORD_API_BASE_URL,
            application_id,
            guild_id
        );

        reqwest::Client::new()
            .post(url)
            .json(command)
            .send()
            .await.unwrap();
    });
    next.run(req).await
}