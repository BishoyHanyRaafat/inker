use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::process::Command;

/// A single caption segment with timing
#[derive(Debug, Clone)]
pub struct Caption {
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
    /// The caption text (cleaned of markup)
    pub text: String,
}

/// Collection of captions from a video
#[derive(Debug, Clone)]
pub struct Captions {
    pub segments: Vec<Caption>,
}

impl Captions {
    /// Get all text concatenated with spaces
    pub fn full_text(&self) -> String {
        self.segments
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn get_caption_at(&self, time: f64) -> Vec<&Caption> {
        self.segments
            .iter()
            .filter(|c| c.start <= time && c.end >= time)
            .collect()
    }

    /// Convert to WebVTT format string
    pub fn to_vtt(&self) -> String {
        let mut vtt = String::from("WEBVTT\n\n");
        for (i, caption) in self.segments.iter().enumerate() {
            vtt.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
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

/// Facade for fetching YouTube captions (public videos) entirely in-memory
#[derive(Clone)]
pub struct YouTubeCaptions {
    client: Client,
}

impl Default for YouTubeCaptions {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeCaptions {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch captions from a public video
    pub async fn fetch_captions(&self, video_url: &str) -> Result<Captions> {
        // Step 1: Use yt-dlp to get video info as JSON
        let output = Command::new("yt-dlp")
            .args(["-j", "--skip-download", video_url])
            .output()
            .context("Failed to run yt-dlp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("yt-dlp failed: {}", stderr));
        }

        let json_str =
            String::from_utf8(output.stdout).context("Invalid UTF-8 in yt-dlp output")?;
        let video_info: VideoInfo =
            serde_json::from_str(&json_str).context("Failed to parse yt-dlp JSON")?;

        // Step 2: Find the json3 format subtitle URL (prefer English)
        let subtitle_url = video_info
            .find_subtitle_url("json3", "en")
            .context("No subtitles found for this video")?;

        // Step 3: Fetch the json3 content directly
        let json3_content = self
            .client
            .get(&subtitle_url)
            .send()
            .await
            .context("Failed to fetch subtitle URL")?
            .text()
            .await
            .context("Failed to read subtitle content")?;

        // Step 4: Parse json3 into our Caption struct
        let captions = Json3Response::parse(&json3_content)?;

        Ok(captions)
    }
}

/// Root video info from yt-dlp -j output
#[derive(Debug, Deserialize)]
struct VideoInfo {
    #[serde(default)]
    subtitles: SubtitleCollection,
    #[serde(default)]
    automatic_captions: SubtitleCollection,
}

impl VideoInfo {
    /// Find a subtitle URL by format extension, preferring a specific language
    fn find_subtitle_url(&self, ext: &str, preferred_lang: &str) -> Option<String> {
        // Try automatic captions first (more common for most videos)
        if let Some(url) = self.automatic_captions.find_url(ext, Some(preferred_lang)) {
            return Some(url);
        }
        if let Some(url) = self.automatic_captions.find_url(ext, None) {
            return Some(url);
        }

        // Fallback to manual subtitles
        if let Some(url) = self.subtitles.find_url(ext, Some(preferred_lang)) {
            return Some(url);
        }
        self.subtitles.find_url(ext, None)
    }
}

/// Collection of subtitles across multiple languages
#[derive(Debug, Default)]
struct SubtitleCollection {
    languages: Vec<LanguageSubtitles>,
}

impl SubtitleCollection {
    /// Find a subtitle URL by format, optionally filtering by language
    fn find_url(&self, ext: &str, lang: Option<&str>) -> Option<String> {
        for lang_subs in &self.languages {
            if let Some(target_lang) = lang
                && lang_subs.language != target_lang
            {
                continue;
            }
            if let Some(format) = lang_subs.formats.iter().find(|f| f.ext == ext) {
                return Some(format.url.clone());
            }
        }
        None
    }
}

/// Subtitles for a specific language
#[derive(Debug)]
struct LanguageSubtitles {
    language: String,
    formats: Vec<SubtitleFormat>,
}

/// A single subtitle format (e.g., vtt, json3, srv1)
#[derive(Debug, Deserialize)]
struct SubtitleFormat {
    ext: String,
    url: String,
}

// Custom deserializer for SubtitleCollection since YouTube returns { "en": [...], "es": [...] }
impl<'de> Deserialize<'de> for SubtitleCollection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};

        struct SubtitleCollectionVisitor;

        impl<'de> Visitor<'de> for SubtitleCollectionVisitor {
            type Value = SubtitleCollection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map of language codes to subtitle formats")
            }

            fn visit_map<M>(self, mut map: M) -> std::result::Result<SubtitleCollection, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut languages = Vec::new();

                while let Some((lang, formats)) = map.next_entry::<String, Vec<SubtitleFormat>>()? {
                    languages.push(LanguageSubtitles {
                        language: lang,
                        formats,
                    });
                }

                Ok(SubtitleCollection { languages })
            }
        }

        deserializer.deserialize_map(SubtitleCollectionVisitor)
    }
}

/// YouTube's json3 caption response
#[derive(Debug, Deserialize)]
struct Json3Response {
    #[serde(default)]
    events: Vec<Json3Event>,
}

impl Json3Response {
    /// Parse json3 content into Captions
    fn parse(json_content: &str) -> Result<Captions> {
        let response: Json3Response =
            serde_json::from_str(json_content).context("Failed to parse json3")?;

        let segments = response
            .events
            .into_iter()
            .filter_map(|event| event.into_caption())
            .collect();

        Ok(Captions { segments })
    }
}

/// A single event in json3 (represents one caption cue)
#[derive(Debug, Deserialize)]
struct Json3Event {
    #[serde(rename = "tStartMs", default)]
    start_ms: u64,
    #[serde(rename = "dDurationMs", default)]
    duration_ms: u64,
    #[serde(default)]
    segs: Vec<Json3Segment>,
}

impl Json3Event {
    /// Convert to Caption if this event has valid text
    fn into_caption(self) -> Option<Caption> {
        let text: String = self
            .segs
            .into_iter()
            .filter_map(|s| s.utf8)
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        // Skip empty or whitespace-only segments
        if text.is_empty() || text == "\n" {
            return None;
        }

        Some(Caption {
            start: self.start_ms as f64 / 1000.0,
            end: (self.start_ms + self.duration_ms) as f64 / 1000.0,
            text,
        })
    }
}

/// A text segment within a json3 event
#[derive(Debug, Deserialize)]
struct Json3Segment {
    utf8: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_captions() {
        let facade = YouTubeCaptions::new();
        let captions = facade
            .fetch_captions("https://www.youtube.com/watch?v=E3_95BZYIVs")
            .await
            .unwrap();

        assert!(
            !captions.segments.is_empty(),
            "Captions should not be empty"
        );

        println!("Found {} caption segments", captions.segments.len());
        for cap in captions.segments.iter().take(5) {
            println!("[{:.2}–{:.2}] {}", cap.start, cap.end, cap.text);
        }

        // Test VTT generation
        let vtt = captions.to_vtt();
        assert!(vtt.starts_with("WEBVTT"));
        println!(
            "\n--- Generated VTT (first 500 chars) ---\n{}",
            &vtt[..500.min(vtt.len())]
        );
    }

    #[test]
    fn test_subtitle_collection_deserialize() {
        let json = r#"{
            "en": [{"ext": "json3", "url": "http://example.com/en.json3"}],
            "es": [{"ext": "vtt", "url": "http://example.com/es.vtt"}]
        }"#;

        let collection: SubtitleCollection = serde_json::from_str(json).unwrap();
        assert_eq!(collection.languages.len(), 2);

        let en_url = collection.find_url("json3", Some("en"));
        assert_eq!(en_url, Some("http://example.com/en.json3".to_string()));

        let es_url = collection.find_url("vtt", Some("es"));
        assert_eq!(es_url, Some("http://example.com/es.vtt".to_string()));
    }
}
