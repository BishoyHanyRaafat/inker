use axum::{
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use std::{
    future::pending,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tracing::{error, info};

use agentic_pipeline::{AgenticPipeline, QuestionedExtractedConcept, pipeline::SectionQuestions};

use super::events::{YouTubeStreamEventUpdate, YouTubeStreamResponse, YouTubeStreamState};
use crate::{AppState, routes::extractors::WsClaims};

#[derive(Debug)]
struct ScheduledQuestion {
    trigger_at: f64,
    questions: SectionQuestions,
}

type GenerationResult = (u64, Result<Vec<QuestionedExtractedConcept>, String>);

#[utoipa::path(
    get,
    path = "/api/v1/interactive/yt/ws",
    tags = ["interactive", "ws"],
    security(("bearer" = [])),
    request_body(
        content = YouTubeStreamEventUpdate,
        description = "Event messages sent over WebSocket",
        content_type = "application/json"
    ),
    responses((
        status = 200,
        description = "WebSocket messages",
        body = YouTubeStreamResponse,
        content_type = "application/json"
    ))
)]
/// WebSocket handler for YouTube stream processing.
///
/// Timestamped outline nodes are scheduled at 80% of their section duration.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    WsClaims(_claims): WsClaims,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    ws.protocols(["inker"])
        .on_upgrade(move |socket| handle_socket(socket, addr, app_state))
}

async fn send_message(socket: &mut WebSocket, msg: YouTubeStreamResponse) -> bool {
    info!("Sending {:?}", msg);
    match serde_json::to_string(&msg) {
        Ok(json) => match socket.send(Message::Text(json.into())).await {
            Ok(()) => true,
            Err(err) => {
                error!("Failed to send WebSocket message: {err}");
                false
            }
        },
        Err(err) => {
            error!("Failed to serialize WebSocket response: {err}");
            false
        }
    }
}

async fn send_error(socket: &mut WebSocket, message: &str, fatal: bool) -> bool {
    send_message(
        socket,
        YouTubeStreamResponse::Error(crate::errors::YoutubeError::Message(
            message.to_owned(),
            fatal,
        )),
    )
    .await
}

/// Fetches captions and generates the outline without blocking the WebSocket loop.
async fn generate_outline(
    app_state: AppState,
    stream_id: String,
) -> Result<Vec<QuestionedExtractedConcept>, String> {
    let transcript = app_state
        .youtube_captions
        .fetch_captions(&stream_id)
        .await
        .map_err(|err| {
            error!("Caption fetching failed for {stream_id}: {err}");
            "Unable to retrieve YouTube captions.".to_owned()
        })?;

    let pipeline = AgenticPipeline::new(transcript)
        .await
        .map_err(|e| e.to_string())?;

    let outline = pipeline.run().await.map_err(|e| e.to_string())?;
    Ok(outline)
}

/// Collects timestamped nodes and sorts them by their 80% playback threshold.
fn build_schedule(nodes: &[QuestionedExtractedConcept]) -> Vec<ScheduledQuestion> {
    let mut schedule = Vec::new();

    for node in nodes {
        let start = node.extracted_concept.meta.start_time;
        let end = node.extracted_concept.meta.end_time;

        let duration = end - start;
        let trigger_at = start + duration * 0.8;

        if start.is_finite()
            && end.is_finite()
            && duration.is_finite()
            && duration > 0.0
            && trigger_at.is_finite()
            && trigger_at >= 0.0
        {
            schedule.push(ScheduledQuestion {
                trigger_at,
                questions: node.section_question.clone(),
            });
        }
    }

    // schedule.sort_by(|a, b| a.trigger_at.total_cmp(&b.trigger_at));
    // They are sorted

    info!("scheduled questions {:?}", schedule);

    schedule
}

/// Estimates playback position using server elapsed time.
/// Periodic frontend position updates are needed to correct buffering/rate drift.
fn current_position(state: &YouTubeStreamState) -> Option<f64> {
    match state {
        YouTubeStreamState::Running { position, instant } => {
            let current = *position + instant.elapsed().as_secs_f64();
            current.is_finite().then_some(current)
        }
        YouTubeStreamState::Paused { .. } | YouTubeStreamState::NotRunning => None,
    }
}

fn valid_playback_position(position: f64) -> bool {
    position.is_finite() && position >= 0.0
}

/// Cap exceptionally large delays and recalculate after waking, avoiding an
/// overflow when Tokio computes the sleep deadline.
fn next_question_delay(
    state: &YouTubeStreamState,
    schedule: &[ScheduledQuestion],
    next_question_idx: usize,
) -> Option<Duration> {
    let position = current_position(state)?;
    let question = schedule.get(next_question_idx)?;
    let seconds = (question.trigger_at - position).max(0.0);

    Some(Duration::try_from_secs_f64(seconds).unwrap_or(Duration::from_secs(60 * 60)))
}

async fn wait_for_next_question(delay: Option<Duration>) {
    match delay {
        Some(duration) => sleep(duration).await,
        None => pending::<()>().await,
    }
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, app_state: AppState) {
    let mut yt_state = YouTubeStreamState::NotRunning;
    let mut schedule = Vec::<ScheduledQuestion>::new();
    let mut next_question_idx = 0usize;

    // A new Start invalidates any older in-flight generation task.
    let mut generation_id = 0u64;
    let (generation_tx, mut generation_rx) = mpsc::unbounded_channel::<GenerationResult>();
    let mut generation_task: Option<JoinHandle<()>> = None;

    'connection: loop {
        // Deliver every question currently due. Each entry is delivered at most once.
        if let Some(position) = current_position(&yt_state) {
            while let Some(question) = schedule.get(next_question_idx) {
                if position < question.trigger_at {
                    break;
                }

                if !send_message(&mut socket, question.questions.clone().into()).await {
                    error!("Failed to send scheduled node to {who}");
                    break 'connection;
                }

                next_question_idx += 1;
            }
        }

        let delay = next_question_delay(&yt_state, &schedule, next_question_idx);

        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let event = match serde_json::from_str::<YouTubeStreamEventUpdate>(&text) {
                            Ok(event) => event,
                            Err(err) => {
                                error!("Invalid WebSocket event from {who}: {err}");
                                if !send_error(&mut socket, "Invalid WebSocket event", false).await {
                                    break 'connection;
                                }
                                continue;
                            }
                        };

                        info!("received {:?}", event);
                        match event {
                            YouTubeStreamEventUpdate::Start { id: stream_id } => {
                                generation_id = generation_id.wrapping_add(1);
                                let this_generation = generation_id;

                                if let Some(task) = generation_task.take() {
                                    task.abort();
                                }

                                // Do not run the old schedule while the new stream is preparing.
                                yt_state = YouTubeStreamState::NotRunning;
                                schedule.clear();
                                next_question_idx = 0;

                                let tx = generation_tx.clone();
                                let state = app_state.clone();

                                generation_task = Some(tokio::spawn(async move {
                                    let result = generate_outline(state, stream_id).await;
                                    // If the connection has closed, sending simply fails.
                                    let _ = tx.send((this_generation, result));
                                }));
                            }

                            YouTubeStreamEventUpdate::Paused => {
                                if let YouTubeStreamState::Running { position, instant } = &yt_state {
                                    let paused_position = *position + instant.elapsed().as_secs_f64();
                                    if paused_position.is_finite() {
                                        yt_state = YouTubeStreamState::Paused {
                                            position: paused_position,
                                        };
                                    }
                                }
                            }

                            YouTubeStreamEventUpdate::Resumed => {
                                if let YouTubeStreamState::Paused { position } = &yt_state {
                                    yt_state = YouTubeStreamState::Running {
                                        position: *position,
                                        instant: Instant::now(),
                                    };
                                }
                            }

                            YouTubeStreamEventUpdate::Seeked { position } => {
                                if !valid_playback_position(position) {
                                    if !send_error(
                                        &mut socket,
                                        "Playback position must be a finite, non-negative number.",
                                        false,
                                    ).await {
                                        break 'connection;
                                    }
                                    continue;
                                }

                                match &yt_state {
                                    YouTubeStreamState::Running { .. } => {
                                        yt_state = YouTubeStreamState::Running {
                                            position,
                                            instant: Instant::now(),
                                        };
                                    }
                                    YouTubeStreamState::Paused { .. } => {
                                        yt_state = YouTubeStreamState::Paused { position };
                                    }
                                    YouTubeStreamState::NotRunning => {}
                                }
                            }
                            YouTubeStreamEventUpdate::PositionUpdated { position } => {
                                if position.is_finite() && position >= 0.0 &&
                                        matches!(yt_state, YouTubeStreamState::Running { .. }) {
                                        yt_state = YouTubeStreamState::Running {
                                            position,
                                            instant: Instant::now(),
                                    };
                                }
                            }
                        }
                    }

                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break 'connection;
                        }
                    }
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Binary(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break 'connection,
                    Some(Err(err)) => {
                        error!("WebSocket error with {who}: {err:?}");
                        break 'connection;
                    }
                }
            }

            generated = generation_rx.recv() => {
                let Some((result_generation, result)) = generated else {
                    error!("Outline generation channel closed for {who}");
                    break 'connection;
                };

                // Ignore results from a stream that has since been replaced.
                if result_generation != generation_id {
                    continue;
                }

                generation_task = None;

                match result {
                    Ok(outline) => {
                        schedule = build_schedule(&outline);
                        next_question_idx = 0;


                        // Wait for the frontend to actually start playback.
                        yt_state = YouTubeStreamState::Paused {
                            position: 0.0,
                        };

                        if !send_message(
                            &mut socket,
                            YouTubeStreamResponse::Ready,
                        ).await {
                            break 'connection;
                        }

                        info!(
                            "Stream ready with {} scheduled questions",
                            schedule.len()
                        );
                    }

                    Err(client_message) => {
                        yt_state = YouTubeStreamState::NotRunning;
                        schedule.clear();
                        next_question_idx = 0;

                        if !send_error(&mut socket, &client_message, true).await {
                            break 'connection;
                        }
                    }
                }
            }

            _ = wait_for_next_question(delay) => {},
        }
    }

    if let Some(task) = generation_task {
        task.abort();
    }

    info!("WebSocket connection closed with {who}");
}
