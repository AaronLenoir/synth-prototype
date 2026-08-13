use serde::Deserialize;

use crate::config::sequencer::{musical_position::MusicalPosition, pattern_config::PatternConfig};

#[derive(Deserialize, PartialEq, Debug)]
pub struct ClipConfig {
    pub start: MusicalPosition,
    pub end: MusicalPosition,
    pub target: String,
    pub pattern: PatternConfig,
}