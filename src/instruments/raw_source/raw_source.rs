use crate::core::{commands::{InstrumentCommand, ParameterId}, instrument::{instrument::Instrument, instrument_error::InstrumentError, instrument_info::InstrumentInfo, instrument_ports::{InstrumentPorts, PortId, PortResolver}}};

// Define the Instrument
pub struct RawSource {
    info: InstrumentInfo,
    ports: InstrumentPorts,

    pub frequency: f32,
    pub waveform: u32,
}

// Define the Ports
pub struct RawSourcePorts;

impl RawSourcePorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
    // Control signal
    pub const IN_CV: PortId = 2;
}

// Define the Parameters (if any) to be used in the set command

pub struct RawSourceParameters;

impl RawSourceParameters {
    pub const FREQUENCY: ParameterId = ParameterId(0);
    pub const WAVEFORM: ParameterId = ParameterId(1);
}

// Implement the PortResolver
impl PortResolver for RawSource {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(RawSourcePorts::OUT_LEFT),
            "OUT_RIGHT" => Some(RawSourcePorts::OUT_RIGHT),
            _ => None,
        }
    }

    fn input_port(&self, name: &str) -> Option<PortId> {
        match name {
            "IN_CV" => Some(RawSourcePorts::IN_CV),
            _ => None,
        }
    }
}

// Implement the constructor
impl RawSource {
    pub fn new(name: &str, frequency: f32, waveform: u32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(1, 2),
            frequency: frequency,
            waveform: waveform,
        }
    }
}

// Implement Instrument trait

impl Instrument for RawSource {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), InstrumentError> {
        for _ in 0..sample_count {
            for port in [RawSourcePorts::OUT_LEFT, RawSourcePorts::OUT_RIGHT] {
                let name = self.info.name().to_owned();
                let output = self.ports.output_port_mut(port);
                output
                    .write_if_connected(0.0)
                    .map_err(|e| InstrumentError::from_port_error(&name, e))?;
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, command: InstrumentCommand) {
        match command {
            InstrumentCommand::Set(RawSourceParameters::FREQUENCY, x) => {
                self.frequency = x;
            },
            InstrumentCommand::Set(RawSourceParameters::WAVEFORM, x) => {
                if x > 0.0 {
                    self.waveform = x as u32;
                }
            }
            _ => {}
        }
    }
}