use serde::Deserialize;

use crate::{
    config::{musical_position::MusicalPosition, pattern_config::PatternConfig},
    sequencer::timeline_position::TimelinePosition,
};

#[derive(Deserialize, PartialEq, Debug)]
pub struct ClipConfig {
    pub start: MusicalPosition,
    pub end: MusicalPosition,
    pub target: String,
    pub pattern: PatternConfig,
}