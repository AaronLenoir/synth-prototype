use crate::{core::commands::InstrumentCommand, sequencer::duration::Duration};

pub struct Pattern {
    pub period: Duration,
    pub commands: Vec<InstrumentCommand>,
}
