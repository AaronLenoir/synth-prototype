use crate::core::{
    commands::{InstrumentCommand, ParameterId},
    instrument::Instrument,
    instrument_error::InstrumentError,
    instrument_info::InstrumentInfo,
    instrument_ports::{InstrumentPorts, PortId, PortResolver},
};

pub const TABLE_SIZE: usize = 100;

pub static SINE_TABLE: [f32; TABLE_SIZE] = [
    0.0000000000,
    0.0627905195,
    0.1253332336,
    0.1873813146,
    0.2486898872,
    0.3090169944,
    0.3681245527,
    0.4257792916,
    0.4817536741,
    0.5358267950,
    0.5877852523,
    0.6374239897,
    0.6845471059,
    0.7289686274,
    0.7705132428,
    0.8090169944,
    0.8443279255,
    0.8763066800,
    0.9048270525,
    0.9297764859,
    0.9510565163,
    0.9685831611,
    0.9822872507,
    0.9921147013,
    0.9980267284,
    1.0000000000,
    0.9980267284,
    0.9921147013,
    0.9822872507,
    0.9685831611,
    0.9510565163,
    0.9297764859,
    0.9048270525,
    0.8763066800,
    0.8443279255,
    0.8090169944,
    0.7705132428,
    0.7289686274,
    0.6845471059,
    0.6374239897,
    0.5877852523,
    0.5358267950,
    0.4817536741,
    0.4257792916,
    0.3681245527,
    0.3090169944,
    0.2486898872,
    0.1873813146,
    0.1253332336,
    0.0627905195,
    0.0000000000,
    -0.0627905195,
    -0.1253332336,
    -0.1873813146,
    -0.2486898872,
    -0.3090169944,
    -0.3681245527,
    -0.4257792916,
    -0.4817536741,
    -0.5358267950,
    -0.5877852523,
    -0.6374239897,
    -0.6845471059,
    -0.7289686274,
    -0.7705132428,
    -0.8090169944,
    -0.8443279255,
    -0.8763066800,
    -0.9048270525,
    -0.9297764859,
    -0.9510565163,
    -0.9685831611,
    -0.9822872507,
    -0.9921147013,
    -0.9980267284,
    -1.0000000000,
    -0.9980267284,
    -0.9921147013,
    -0.9822872507,
    -0.9685831611,
    -0.9510565163,
    -0.9297764859,
    -0.9048270525,
    -0.8763066800,
    -0.8443279255,
    -0.8090169944,
    -0.7705132428,
    -0.7289686274,
    -0.6845471059,
    -0.6374239897,
    -0.5877852523,
    -0.5358267950,
    -0.4817536741,
    -0.4257792916,
    -0.3681245527,
    -0.3090169944,
    -0.2486898872,
    -0.1873813146,
    -0.1253332336,
    -0.0627905195,
];

pub struct SignalSourcePorts;

impl SignalSourcePorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
}

impl PortResolver for SignalSource {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(SignalSourcePorts::OUT_LEFT),
            "OUT_RIGHT" => Some(SignalSourcePorts::OUT_RIGHT),
            _ => None,
        }
    }

    fn input_port(&self, _name: &str) -> Option<PortId> {
        None
    }
}

pub struct SignalSourceParameters;

impl SignalSourceParameters {
    pub const FREQUENCY: ParameterId = ParameterId(0);
}

pub struct SignalSource {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    frequency: f32,
    phase: f32,
}

impl SignalSource {
    pub fn new(name: &str, frequency: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(0, 2),
            frequency: frequency,
            phase: 0.0,
        }
    }

    fn next_value(&mut self, delta: f32) -> f32 {
        self.phase += delta;

        while self.phase >= 1.0 {
            self.phase = self.phase - 1.0;
        }

        let position = TABLE_SIZE as f32 * self.phase;
        let index = position as usize;
        let frac = position.fract();

        // Interpolate if we're between two data points
        let a = SINE_TABLE[index];
        let b = SINE_TABLE[(index + 1) % TABLE_SIZE];

        a + (b - a) * frac
    }
}

impl Instrument for SignalSource {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), InstrumentError> {
        let wave_length = 1_000_000_000.0 / self.frequency;
        let sample_length = time_window as f32 / sample_count as f32;
        let delta = sample_length / wave_length;

        for _ in 0..sample_count {
            let sample = self.next_value(delta);

            for port in [SignalSourcePorts::OUT_LEFT, SignalSourcePorts::OUT_RIGHT] {
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
            InstrumentCommand::Set(SignalSourceParameters::FREQUENCY, x) => {
                self.frequency = x;
            }
            _ => {}
        }
    }
}
