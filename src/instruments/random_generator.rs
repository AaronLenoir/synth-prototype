use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::core::{
    instrument::Instrument,
    instrument_error::InstrumentError,
    instrument_info::InstrumentInfo,
    instrument_ports::{InstrumentPorts, PortId, PortResolver},
};

pub struct RandomGeneratorPorts;

impl RandomGeneratorPorts {
    pub const OUT: PortId = 0;
}

impl PortResolver for RandomGenerator {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT" => Some(RandomGeneratorPorts::OUT),
            _ => None,
        }
    }

    fn input_port(&self, _name: &str) -> Option<PortId> {
        None
    }
}

pub struct RandomGenerator {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    rng: StdRng,
}

impl RandomGenerator {
    pub fn new(name: &str) -> Self {
        let rng = StdRng::seed_from_u64(0);

        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(0, 1),
            rng: rng,
        }
    }
}

impl Instrument for RandomGenerator {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), InstrumentError> {
        let Self { rng, ports, .. } = self;
        let name = self.info.name().to_owned();

        let out = ports.output_port_mut(RandomGeneratorPorts::OUT);

        if out.is_connected() {
            for _ in 0..sample_count {
                let value: f32 = rng.random_range(-1.0..1.0);
                out.write_if_connected(value)
                    .map_err(|e| InstrumentError::from_port_error(&name, e))?;
            }
        }

        Ok(())
    }
}
