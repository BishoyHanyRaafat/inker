use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use url::Url;
use yt_transcript_rs::YouTubeTranscriptApi;

/// A single caption segment with timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caption {
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds.
    pub end: f64,
    /// Caption text.
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Transcript {
    pub video_id: String,
    pub captions: Captions,
    pub title: String,
    pub description: String,
}

impl Transcript {
    pub fn new(
        video_id: impl Into<String>,
        captions: Captions,
        title: String,
        description: String,
    ) -> Self {
        Self {
            video_id: video_id.into(),
            captions,
            title,
            description,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.captions.segments.is_empty()
    }
}

/// Collection of captions from a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Captions {
    pub segments: Vec<Caption>,
}

impl Captions {
    /// Get all text concatenated with spaces.
    pub fn full_text(&self) -> String {
        self.segments
            .iter()
            .map(|caption| caption.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get captions overlapping the requested time range.
    pub fn query(&self, q_start: f64, q_end: f64) -> Captions {
        if q_end <= q_start {
            return Captions {
                segments: Vec::new(),
            };
        }

        let left = self
            .segments
            .partition_point(|caption| caption.end <= q_start);

        let mut right = left;

        while right < self.segments.len() && self.segments[right].start < q_end {
            right += 1;
        }

        Captions {
            segments: self.segments[left..right].to_vec(),
        }
    }

    /// Convert to WebVTT format.
    pub fn to_vtt(&self) -> String {
        let mut vtt = String::from("WEBVTT\n\n");

        for (index, caption) in self.segments.iter().enumerate() {
            vtt.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                index + 1,
                format_vtt_time(caption.start),
                format_vtt_time(caption.end),
                caption.text
            ));
        }

        vtt
    }
}

fn format_vtt_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = seconds % 60.0;

    format!("{:02}:{:02}:{:06.3}", hours, minutes, secs)
}

/// Extract a YouTube video ID from a URL or accept a raw video ID.
fn extract_video_id(input: &str) -> Result<String> {
    let input = input.trim();

    // Allow callers to pass the video ID directly.
    if !input.contains("://") && !input.contains('/') && !input.contains('?') {
        return validate_video_id(input);
    }

    let url = Url::parse(input).context("Invalid YouTube URL")?;

    let host = url.host_str().ok_or_else(|| anyhow!("Missing URL host"))?;

    let video_id = match host {
        "youtu.be" => url
            .path_segments()
            .and_then(|mut segments| segments.next().map(|e| e.to_string())),

        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com" => {
            // Standard watch URL.
            if let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "v") {
                Some(value.into_owned())
            } else {
                // Shorts, embed, and live URLs.
                let segments: Vec<_> = url.path_segments().into_iter().flatten().collect();

                segments
                    .windows(2)
                    .find(|window| matches!(window[0], "shorts" | "embed" | "live"))
                    .map(|window| window[1].to_string())
            }
        }

        _ => return Err(anyhow!("Unsupported YouTube URL host: {host}")),
    };

    validate_video_id(
        video_id
            .as_deref()
            .ok_or_else(|| anyhow!("Could not extract YouTube video ID"))?,
    )
}

fn validate_video_id(video_id: &str) -> Result<String> {
    let valid = video_id.len() == 11
        && video_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');

    if !valid {
        return Err(anyhow!("Invalid YouTube video ID"));
    }

    Ok(video_id.to_string())
}

/// Facade for fetching YouTube captions and metadata.
#[derive(Clone)]
pub struct YouTubeCaptions {
    client: YouTubeTranscriptApi,
}

impl YouTubeCaptions {
    /// Initialize the YouTube transcript client.
    pub fn new() -> Result<Self> {
        let client = YouTubeTranscriptApi::new(None, None, None)
            .context("Failed to initialize YouTube transcript API")?;
        Ok(Self { client })
    }

    /// Fetch captions and video metadata.
    ///
    /// Supports both YouTube URLs and raw video IDs.
    pub async fn fetch_captions(&self, video_url: &str) -> Result<Transcript> {
        let video_id = extract_video_id(video_url)?;

        // Fetch transcript and metadata concurrently.
        let (fetched, details) = tokio::try_join!(
            self.client.fetch_transcript(&video_id, &["en"], false,),
            self.client.fetch_video_details(&video_id),
        )
        .context("Failed to fetch YouTube transcript or metadata")?;

        // Convert library snippets into application captions.
        let segments = fetched
            .snippets
            .into_iter()
            .filter(|snippet| !snippet.text.trim().is_empty())
            .map(|snippet| Caption {
                start: snippet.start,
                end: snippet.start + snippet.duration,
                text: snippet.text.trim().to_string(),
            })
            .collect::<Vec<_>>();

        if segments.is_empty() {
            return Err(anyhow!("YouTube transcript contains no caption segments"));
        }

        Ok(Transcript {
            video_id: fetched.video_id,
            captions: Captions { segments },
            title: details.title,
            description: details.short_description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        let cases = [
            ("https://www.youtube.com/watch?v=E3_95BZYIVs", "E3_95BZYIVs"),
            ("https://youtu.be/E3_95BZYIVs", "E3_95BZYIVs"),
            ("https://www.youtube.com/shorts/E3_95BZYIVs", "E3_95BZYIVs"),
            ("https://www.youtube.com/embed/E3_95BZYIVs", "E3_95BZYIVs"),
            ("E3_95BZYIVs", "E3_95BZYIVs"),
        ];

        for (input, expected) in cases {
            assert_eq!(extract_video_id(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_invalid_video_id() {
        assert!(extract_video_id("invalid").is_err());
    }

    #[test]
    fn test_caption_query() {
        let captions = Captions {
            segments: vec![
                Caption {
                    start: 0.0,
                    end: 2.0,
                    text: "First".into(),
                },
                Caption {
                    start: 2.0,
                    end: 4.0,
                    text: "Second".into(),
                },
                Caption {
                    start: 4.0,
                    end: 6.0,
                    text: "Third".into(),
                },
            ],
        };

        let result = captions.query(2.0, 5.0);

        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.segments[0].text, "Second");
        assert_eq!(result.segments[1].text, "Third");
    }

    #[test]
    fn test_full_text() {
        let captions = Captions {
            segments: vec![
                Caption {
                    start: 0.0,
                    end: 2.0,
                    text: "Hello".into(),
                },
                Caption {
                    start: 2.0,
                    end: 4.0,
                    text: "world".into(),
                },
            ],
        };

        assert_eq!(captions.full_text(), "Hello world");
    }

    #[test]
    fn test_vtt_generation() {
        let captions = Captions {
            segments: vec![Caption {
                start: 1.0,
                end: 3.0,
                text: "Hello world".into(),
            }],
        };

        let vtt = captions.to_vtt();

        assert!(vtt.starts_with("WEBVTT"));
        assert!(vtt.contains("00:00:01.000 --> 00:00:03.000"));
        assert!(vtt.contains("Hello world"));
    }

    #[tokio::test]
    #[ignore = "Requires network access and YouTube availability"]
    async fn test_fetch_captions() {
        let facade = YouTubeCaptions::new().unwrap();

        let transcript = facade
            .fetch_captions("https://www.youtube.com/watch?v=K5KVEU3aaeQ")
            .await
            .unwrap();

        assert!(!transcript.is_empty());
        assert!(!transcript.title.is_empty());
        assert!(!transcript.description.is_empty());

        println!("Title: {}", transcript.title);
        println!("Description: {}", transcript.description);
        println!("Video ID: {}", transcript.video_id);

        println!(
            "Found {} caption segments",
            transcript.captions.segments.len()
        );

        for caption in transcript.captions.segments.iter().take(5) {
            println!("[{:.2}–{:.2}] {}", caption.start, caption.end, caption.text);
        }
    }
}
