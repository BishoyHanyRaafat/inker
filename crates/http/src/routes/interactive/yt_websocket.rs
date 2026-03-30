use axum::{
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::time::interval;
use tracing::{error, info};

use super::events::{YouTubeStreamEventUpdate, YouTubeStreamResponse, YouTubeStreamState};
use crate::{AppState, routes::extractors::WsClaims};

#[utoipa::path(
    get,
    path = "/api/v1/interactive/yt/ws",
    tags = ["interactive", "ws"],
    security(
        ("bearer" = [])
    ),
    request_body(content = YouTubeStreamEventUpdate, description = "Event messages sent over WebSocket", content_type = "application/json"),
responses(
    (status = 200, description = "WebSocket messages", body = YouTubeStreamResponse, content_type = "application/json")
    )
)]
/// Websocket handler for YouTube stream processing
/// Every 10 seconds, sends processed chunks based on the current stream state
/// YouTubeStreamEventUpdate messages control the stream state
/// You should send updates as the user interacts with the stream
/// Make sure the access token stored in the cookies
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    WsClaims(_claims): WsClaims,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // If the browser supplies `Sec-WebSocket-Protocol`, it expects the server to select one.
    // We always select the stable "inker" protocol (the JWT is also provided as an offered
    // protocol value and is parsed by `WsClaims` from the request headers).
    ws.protocols(["inker"])
        .on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

// Helper to safely send a message
async fn send_message(socket: &mut WebSocket, msg: YouTubeStreamResponse) -> bool {
    info!("Sending message: {:?}", msg);
    match serde_json::to_string(&msg) {
        Ok(json) => socket.send(Message::Text(json.into())).await.is_ok(),
        Err(err) => {
            error!("Failed to serialize YouTubeStreamResponse::Error: {}", err);
            false
        }
    }
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, app_state: AppState) {
    let mut ticker = interval(Duration::from_secs(20));
    let mut yt_state = YouTubeStreamState::NotRunning;
    let mut captions: Option<yt_processing::Captions> = None;

    loop {
        tokio::select! {
        // Receive messages
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(t))) => {
                    info!("Received message from {}: {}", who, t);
                    // Parse the event
                    let event = match serde_json::from_str::<YouTubeStreamEventUpdate>(&t) {
                        Ok(event) => event,
                        Err(e) => {
                            if !send_message(&mut socket, e.into()).await {
                                break;
                            }
                            continue;
                        }
                    };

                    // Handle the event
                    match event {
                        YouTubeStreamEventUpdate::Start { id: stream_id } => {
                            captions = app_state.youtube_captions.fetch_captions(&stream_id).await.ok();
                            yt_state = YouTubeStreamState::Running {
                                position: 0.0,
                                instant: Instant::now(),
                            };
                            ticker.tick().await;
                        }
                        YouTubeStreamEventUpdate::Paused => {
                            yt_state = match yt_state {
                                YouTubeStreamState::Running { position, instant } => {
                                    let elapsed = instant.elapsed().as_secs_f64();
                                    YouTubeStreamState::Paused { position: position + elapsed }
                                }
                                _ => yt_state,
                            };
                        }
                        YouTubeStreamEventUpdate::Resumed => {
                            yt_state = match yt_state {
                                YouTubeStreamState::Paused { position } => {
                                    YouTubeStreamState::Running { position, instant: Instant::now() }
                                }
                                _ => yt_state,
                            };
                        }
                        YouTubeStreamEventUpdate::Seeked { position } => {
                            yt_state = match yt_state {
                                YouTubeStreamState::Running { .. } => {
                                    YouTubeStreamState::Running { position, instant: Instant::now() }
                                }
                                YouTubeStreamState::Paused { .. } => {
                                    YouTubeStreamState::Paused { position }
                                }
                                YouTubeStreamState::NotRunning => yt_state,
                            };
                        }
                    }
                }
                Some(Ok(Message::Binary(_))) => {} // ignore binary
                Some(Ok(Message::Close(_))) => break,
                Some(Err(e)) => {
                    error!("WebSocket error with {}: {:?}", who, e);
                    break;
                }
                None => break,
                _ => {}
            },

            // Inside the ticker tick branch
            _ = ticker.tick() => {
                if let YouTubeStreamState::Running { position, instant } = &mut yt_state {
                    let elapsed = instant.elapsed().as_secs_f64();
                    let current_postion_begin = *position;
                    let current_position = *position + elapsed;

                    *position = current_position;

                    if let Some(captions_some) = &captions {
                        // Get the text chunk safely
                        let listened = captions_some.query(current_postion_begin, current_position).full_text();

                        // Process the chunk
                        match app_state.gemini_client.process_chunk(listened.into()).await {
                            Ok(processed_chunk) => {
                                if !send_message(&mut socket, processed_chunk.into()).await {
                                    error!("Failed to send processed chunk to {}", who);
                                    break;
                                }
                            }
                            Err(e) => {
                                // Handle processing error
                                if !send_message(&mut socket, e.into()).await {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("WebSocket connection closed with {}", who);
}
