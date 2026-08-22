use std::collections::HashMap;

use crate::{
    core::{
        commands::{InstrumentCommand, ParameterId},
        instrument::{
            instrument::Instrument,
            instrument_error::InstrumentError,
            instrument_info::InstrumentInfo,
            instrument_ports::{InstrumentPorts, PortId, PortResolver},
        },
    },
    instruments::raw_source::waveform::Waveform,
    sequencer::event::RackEvent,
};

/// RawSource is a signal source than can generate a continuous signal in a
/// configurable frequency with several possible waveforms
/// Additionally it features has a CV_IN port that can be used to mod the
/// frequency
pub struct RawSource {
    pub frequency: f32,
    pub waveform: Waveform,
    pub fm_depth: f32,

    info: InstrumentInfo,
    ports: InstrumentPorts,
    phase: f32,
}

// Define the Ports
pub struct RawSourcePorts;

impl RawSourcePorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
    // Control signal
    pub const IN_CV: PortId = 0;
}

// Define the Parameters (if any) to be used in the set command

pub struct RawSourceParameters;

impl RawSourceParameters {
    pub const FREQUENCY: ParameterId = ParameterId(0);
    pub const WAVEFORM: ParameterId = ParameterId(1);
    pub const FM_DEPTH: ParameterId = ParameterId(2);
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
    pub fn new(name: &str, frequency: f32, waveform: u32, fm_depth: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(1, 2),
            frequency: frequency,
            phase: 0.0,
            waveform: RawSource::map_maveform(waveform),
            fm_depth: fm_depth,
        }
    }

    fn map_maveform(waveform: u32) -> Waveform {
        match waveform {
            1 => Waveform::Sine,
            2 => Waveform::Saw,
            3 => Waveform::Square,
            _ => Waveform::None,
        }
    }

    fn next_phase(&self, delta: f32) -> f32 {
        let mut next_phase = self.phase + delta;

        while next_phase >= 1.0 {
            next_phase = next_phase - 1.0;
        }

        next_phase
    }

    fn read_cv_in(&mut self) -> Result<f32, InstrumentError> {
        let name = self.info.name().to_owned();
        let value = self
            .ports
            .input_port_mut(RawSourcePorts::IN_CV)
            .read_if_connected()
            .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        Ok(value)
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

    fn update(
        &mut self,
        time_window: u128,
        sample_count: u32,
        events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        for sample_offset in 0..sample_count {
            self.handle_events_at_sample(sample_offset, &events);

            let mod_value = self.read_cv_in()?;

            let modulated_frequency = self.frequency + (self.frequency * mod_value * self.fm_depth);
            let wave_length = 1_000_000_000.0 / modulated_frequency;
            let sample_length = time_window as f32 / sample_count as f32;
            let phase_delta = sample_length / wave_length;
            self.phase = self.next_phase(phase_delta);

            let sample = self.waveform.sample(self.phase, phase_delta);

            for port in [RawSourcePorts::OUT_LEFT, RawSourcePorts::OUT_RIGHT] {
                let name = self.info.name().to_owned();
                let output = self.ports.output_port_mut(port);
                output
                    .write_if_connected(sample)
                    .map_err(|e| InstrumentError::from_port_error(&name, e))?;
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, command: InstrumentCommand) {
        match command {
            InstrumentCommand::Set(RawSourceParameters::FREQUENCY, x) => {
                if (x > 0.0) {
                    self.frequency = x;
                }
            }
            InstrumentCommand::Set(RawSourceParameters::WAVEFORM, x) => {
                if x > 0.0 {
                    self.waveform = RawSource::map_maveform(x as u32);
                }
            }
            InstrumentCommand::Set(RawSourceParameters::FM_DEPTH, x) => {
                if x >= 0.0 && x <= 1.0 {
                    self.fm_depth = x;
                }
            }
            _ => {}
        }
    }
}
