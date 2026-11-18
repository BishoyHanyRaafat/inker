use agentic_pipeline::{
    AgenticPipeline, MaterialNote, SectionQuestions, pipeline::ExtractedConcept,
};
use axum::{Json, extract::State};
use oauth2::url;

use crate::{
    AppState,
    errors::{AppError, AppErrorTrait, BaseError, YoutubeError},
    responses::ApiResponse,
    routes::{
        auth::models::Claims,
        notes::{
            models::{Note, SeededNote},
            notes_handler::create_note,
        },
        yt_material::models::{
            ScheduledQuestion, YouTubeQuestionsRequest, YouTubeQuestionsResponse,
        },
    },
};
fn extract_video_id(input: &str) -> Result<String, String> {
    let url = url::Url::parse(input).map_err(|_| "Invalid YouTube URL".to_owned())?;

    let host = url.host_str().ok_or("Invalid YouTube URL")?;

    let id = match host {
        "youtu.be" => url.path_segments().and_then(|mut segments| segments.next()),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" => {
            if url.path() == "/watch" {
                url.query_pairs()
                    .find(|(key, _)| key == "v")
                    .map(|(_, value)| value)
                    .map(|v| Box::leak(v.into_owned().into_boxed_str()) as &str)
            } else {
                url.path_segments().and_then(|mut segments| {
                    let first = segments.next()?;

                    match first {
                        "embed" | "shorts" | "live" => segments.next(),
                        _ => None,
                    }
                })
            }
        }
        _ => return Err("Unsupported YouTube URL".to_owned()),
    };

    id.map(str::to_owned)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "Could not extract YouTube video ID".to_owned())
}

fn build_schedule(
    questions: &[SectionQuestions],
    concepts: &[ExtractedConcept],
) -> Vec<ScheduledQuestion> {
    questions
        .iter()
        .zip(concepts.iter())
        .filter_map(|(questions, concept)| {
            let start = concept.meta.start_time;
            let end = concept.meta.end_time;

            let duration = end - start;
            let trigger_at = start + duration * 0.8;

            if !start.is_finite()
                || !end.is_finite()
                || !duration.is_finite()
                || duration <= 0.0
                || !trigger_at.is_finite()
                || trigger_at < 0.0
            {
                return None;
            }

            Some(ScheduledQuestion {
                trigger_at,
                questions: questions.clone(),
            })
        })
        .collect()
}
#[utoipa::path(
    post,
    path = "/api/v1/yt/questions",
    tag = "YouTube",
    security(("bearer" = [])),

    request_body(
        content = YouTubeQuestionsRequest,
        description = "YouTube video URL",
        content_type = "application/json"
    ),
    responses(
        (
            status = 200,
            description = "Successfully generated timestamped questions",
            body = YouTubeQuestionsResponse
        ),
        (
            status = 400,
            description = "Invalid YouTube URL"
        ),
        (
            status = 422,
            description = "Unable to retrieve YouTube captions"
        ),
        (
            status = 500,
            description = "Question generation failed"
        )
    )
)]
#[axum::debug_handler]
pub async fn yt_question_handler(
    _claims: Claims,
    State(app_state): State<AppState>,
    Json(payload): Json<YouTubeQuestionsRequest>,
) -> Result<Json<YouTubeQuestionsResponse>, AppError> {
    let video_id = extract_video_id(&payload.url)
        .map_err(|e| YoutubeError::VideoNotFound.get_response().with_debug(e))?;

    let transcript = app_state
        .youtube_captions
        .fetch_captions(&video_id)
        .await
        .map_err(|e| {
            tracing::error!("Caption fetching failed: {e}");
            BaseError::UnprocessableEntity
                .get_response()
                .with_message("Unable to retrieve YouTube captions.".to_string())
        })?;

    let pipeline = AgenticPipeline::new(transcript).await.map_err(|e| {
        tracing::error!("Pipeline initialization failed: {e}");
        BaseError::InternalServer.get_response().with_debug(e)
    })?;

    let outline = pipeline.extract().await.map_err(|e| {
        tracing::error!("Question generation failed: {e}");
        BaseError::InternalServer.get_response().with_debug(e)
    })?;
    let questions = outline.generate_questions().await.map_err(|e| {
        tracing::error!("Question generation failed: {e}");
        BaseError::InternalServer.get_response().with_debug(e)
    })?;

    let questions = build_schedule(questions.questions(), questions.concepts());

    Ok(Json(YouTubeQuestionsResponse { questions }))
}
#[utoipa::path(
    post,
    path = "/api/v1/yt/summary",
    tag = "YouTube",
    request_body = YouTubeQuestionsRequest,
    responses(
        (
            status = 200,
            description = "YouTube summary generated and saved successfully",
            body = ApiResponse<Note>
        ),
        (
            status = 404,
            description = "YouTube video not found"
        ),
        (
            status = 422,
            description = "Unable to retrieve YouTube captions"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer" = [])
    )
)]
#[axum::debug_handler]
pub async fn yt_summary_handler(
    claims: Claims,
    State(app_state): State<AppState>,
    Json(payload): Json<YouTubeQuestionsRequest>,
) -> Result<ApiResponse<Note>, AppError> {
    let video_id = extract_video_id(&payload.url)
        .map_err(|e| YoutubeError::VideoNotFound.get_response().with_debug(e))?;

    let transcript = app_state
        .youtube_captions
        .fetch_captions(&video_id)
        .await
        .map_err(|e| {
            tracing::error!("Caption fetching failed: {e:#}");
            BaseError::UnprocessableEntity
                .get_response()
                .with_message("Unable to retrieve YouTube captions.".to_string())
        })?;

    let pipeline = AgenticPipeline::new(transcript).await.map_err(|e| {
        tracing::error!("Pipeline initialization failed: {e}");
        BaseError::InternalServer.get_response().with_debug(e)
    })?;

    let extracted = pipeline.extract().await.map_err(|e| {
        tracing::error!("Transcript extraction failed: {e}");
        BaseError::InternalServer.get_response().with_debug(e)
    })?;

    let summary = extracted
        .generate_notes_plan()
        .await
        .map_err(|e| {
            tracing::error!("Summary generation failed: {e}");
            BaseError::InternalServer.get_response().with_debug(e)
        })?
        .generate_notes()
        .await
        .map_err(|e| {
            tracing::error!("Summary generation failed: {e}");
            BaseError::InternalServer.get_response().with_debug(e)
        })?;

    let MaterialNote {
        title,
        sections,
        coverage_notes,
        tags,
        description,
        subject,
        learning_objectives,
    } = summary.into_summary();

    let note = create_note(
        claims,
        axum::extract::State(app_state.clone()),
        axum::Json(SeededNote {
            title,
            archived: Some(false),
            pinned: Some(false),
            version: None,
            content: sections
                .into_iter()
                .map(|section| section.content)
                .collect::<Vec<_>>()
                .join("<br>"),
            coverage_notes: Some(coverage_notes),
            tags: Some(tags),
            description: Some(description),
            subject: Some(subject),
            learning_objectives: Some(learning_objectives),
        }),
    )
    .await?;

    Ok(note)
}
