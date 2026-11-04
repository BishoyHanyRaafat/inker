use yt_processing::Captions;

#[derive(Debug, Clone)]
pub struct Transcript {
    pub video_id: String,
    pub captions: Captions,
}

impl Transcript {
    pub fn new(video_id: impl Into<String>, captions: Captions) -> Self {
        Self {
            video_id: video_id.into(),
            captions,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.captions.segments.is_empty()
    }
}
