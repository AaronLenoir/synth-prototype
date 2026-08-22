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
    sequencer::event::RackEvent,
};

/// Every Instrument must implement this trait
/// At the core is the update function, which is called in the update loop
/// of the application and generates the actual samples and publishes these
/// to the output ports
pub trait Instrument: PortResolver {
    fn info(&self) -> &InstrumentInfo;
    fn ports(&mut self) -> &mut InstrumentPorts;

    /// The update function is called every iteration of the update loop
    /// this function is performance critical as it calls many time and needs
    /// to finish in time to avoid delays in the audio output
    ///
    /// - time_window: indicates for how much real time we are generating audio
    /// - sample_count: indicates how many samples to generate, each instrument MUST generate
    ///                 exactly this amount of samples on each connected port
    /// - events: any events from the sequencer will be provided in a HashMap, indexed
    ///           by the sample update. For each sample the instrument can check if there are
    ///           events to handle, this ensures the events are triggered at sample resolution
    ///           More than one event can occur on a single sample offsset
    ///           It is not mandatory for Instruments to process the events if it does not make
    ///           sense.
    ///           A helper function self.handle_events_at_sample can be used for convenience
    fn update(
        &mut self,
        time_window: u128,
        sample_count: u32,
        events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError>;

    /// Handles a given InstrumentCommand (if it makes sense)
    /// A default implementation does nothing
    fn handle_command(&mut self, _command: InstrumentCommand) {
        // Don't do anything by default ...
    }

    /// Given a sample offset, will check for any events to trigger and calls
    /// handle_command for each
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
