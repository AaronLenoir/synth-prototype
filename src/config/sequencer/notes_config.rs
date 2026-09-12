use std::collections::HashMap;

use serde::Deserialize;

use crate::config::sequencer::musical_position::MusicalPosition;

#[derive(Deserialize, Default, PartialEq, Debug)]
pub struct NotesConfig {
    pub note: u32,
    pub velocity: f32,
    pub start: MusicalPosition,
    pub end: MusicalPosition,
}
