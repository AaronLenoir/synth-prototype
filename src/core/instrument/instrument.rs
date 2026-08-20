use std::collections::HashMap;

use crate::{
    core::{
        commands::{InstrumentCommand, RackCommand},
        instrument::{
            instrument_error::InstrumentError,
            instrument_info::InstrumentInfo,
            instrument_ports::{InstrumentPorts, PortResolver},
        },
    },
    sequencer::{event::RackEvent, sample_offset::SampleOffset},
};

pub trait Instrument: PortResolver {
    fn info(&self) -> &InstrumentInfo;
    fn ports(&mut self) -> &mut InstrumentPorts;

    fn update(
        &mut self,
        time_window: u128,
        sample_count: u32,
        events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError>;

    fn handle_command(&mut self, _command: InstrumentCommand) {
        // Don't do anything by default ...
    }

    fn handle_events_at_sample(
        &mut self,
        sample_offset: u32,
        events: &HashMap<u32, Vec<&RackEvent>>,
    ) {
        let current_events = events.get(&sample_offset);
        if current_events.is_some() {
            for current_event in current_events.unwrap() {
                let RackCommand::Instrument { command, .. } = &current_event.command;
                self.handle_command(*command);
            }
        }
    }
}
