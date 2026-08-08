use crate::core::{
    commands::InstrumentCommand,
    instrument_error::InstrumentError,
    instrument_info::InstrumentInfo,
    instrument_ports::{InstrumentPorts, PortResolver},
};

pub trait Instrument: PortResolver {
    fn info(&self) -> &InstrumentInfo;
    fn ports(&mut self) -> &mut InstrumentPorts;

    fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), InstrumentError>;

    fn handle_command(&mut self, _command: InstrumentCommand) {
        // Don't do anything by default ...
    }
}
