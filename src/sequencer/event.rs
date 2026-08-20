use crate::{
    core::commands::{InstrumentCommand, RackCommand},
    sequencer::{sample_offset::SampleOffset, timeline_position::TimelinePosition},
};

pub struct InstrumentEvent {
    pub position: TimelinePosition,
    pub command: InstrumentCommand,
}

impl InstrumentEvent {
    pub fn new(position: TimelinePosition, command: InstrumentCommand) -> Self {
        Self { position, command }
    }
}

pub struct RackEvent {
    pub offset: SampleOffset,
    pub command: RackCommand,
}

impl RackEvent {
    pub fn new(offset: SampleOffset, command: RackCommand) -> Self {
        Self { offset, command }
    }
}
