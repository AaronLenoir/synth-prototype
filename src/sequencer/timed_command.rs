use crate::{core::commands::InstrumentCommand, sequencer::timeline_position::TimelinePosition};

pub struct TimedCommand {
    pub start: TimelinePosition,
    pub command: InstrumentCommand,
}
