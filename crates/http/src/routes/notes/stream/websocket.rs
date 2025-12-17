use axum::{
    extract::{
        ConnectInfo, State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
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
use crate::AppState;

// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

// Helper to safely send a message
async fn send_message(socket: &mut WebSocket, msg: Utf8Bytes) -> bool {
    socket.send(Message::Text(msg)).await.is_ok()
}

// Helper to send a YoutubeError
async fn send_error(socket: &mut WebSocket, e: impl Into<crate::errors::YoutubeError>) -> bool {
    match serde_json::to_string(&YouTubeStreamResponse::Error(e.into())) {
        Ok(json) => send_message(socket, json.into()).await,
        Err(err) => {
            error!("Failed to serialize YouTubeStreamResponse::Error: {}", err);
            false
        }
    }
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, app_state: AppState) {
    let mut ticker = interval(Duration::from_secs(10));
    let mut yt_state = YouTubeStreamState::NotRunning;
    let mut captions: Option<yt_processing::Captions> = None;

    loop {
        tokio::select! {
            // Receive messages
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(t))) => {
                    // Parse the event
                    let event = match serde_json::from_str::<YouTubeStreamEventUpdate>(&t) {
                        Ok(event) => event,
                        Err(e) => {
                            if !send_error(&mut socket, e).await {
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

            _ = ticker.tick() => {
                // if let (YouTubeStreamState::Running { position, instant }) = &mut yt_state {
                //     let mut elapsed = instant.elapsed().as_secs_f64();
                //     let current_position = *position + elapsed;
                //
                //     // Send captions if available
                //     if let Some(captions) = &captions {
                //         if let Some(caption_text) = captions.get_caption_at(current_position) {
                //             let caption_msg = format!("Caption at {:.2}s: {}", current_position, caption_text);
                //             if !send_message(&mut socket, caption_msg.into()).await {
                //                 break;
                //             }
                //         }
                //     }
                // }
            }
        }
    }

    info!("WebSocket connection closed with {}", who);
}
