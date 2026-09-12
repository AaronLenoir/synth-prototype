use serde::Deserialize;

use crate::config::sequencer::{
    musical_position::MusicalPosition, notes_config::NotesConfig, pattern_config::PatternConfig,
};

#[derive(Deserialize, PartialEq, Debug)]
pub struct ClipConfig {
    pub start: MusicalPosition,
    pub end: MusicalPosition,
    pub target: String,
    #[serde(default)]
    pub pattern: PatternConfig,
    #[serde(default)]
    pub notes: Vec<NotesConfig>,
}
