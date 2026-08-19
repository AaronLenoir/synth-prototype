use crate::{core::commands::InstrumentCommand, sequencer::timeline_position::TimelinePosition};

pub struct Event {
    pub position: TimelinePosition,
    pub command: InstrumentCommand,
}

impl Event {
    pub fn new(position: TimelinePosition, command: InstrumentCommand) -> Self {
        Self {
            position,
            command,
        }
    }
}