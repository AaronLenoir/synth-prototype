use std::collections::HashMap;

use crate::{
    core::instrument::{
        instrument::Instrument,
        instrument_error::InstrumentError,
        instrument_info::InstrumentInfo,
        instrument_ports::{InstrumentPorts, PortId, PortResolver},
    },
    sequencer::event::RackEvent,
};

/// Instrument generating a constant value, used for testing only
pub struct DCGeneratorPorts;

impl DCGeneratorPorts {
    pub const OUT: PortId = 0;
}

impl PortResolver for DCGenerator {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT" => Some(DCGeneratorPorts::OUT),
            _ => None,
        }
    }

    fn input_port(&self, _name: &str) -> Option<PortId> {
        // We have no input ports
        None
    }
}

pub struct DCGenerator {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    value: f32,
}

impl DCGenerator {
    pub fn new(name: &str, value: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(0, 1),
            value: value,
        }
    }
}

impl Instrument for DCGenerator {
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
        let value = self.value;
        let name = self.info.name().to_owned();

        let out = self.ports().output_port_mut(DCGeneratorPorts::OUT);
        for _ in 0..sample_count {
            out.write_if_connected(value)
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod dc_generator_tests {
    use std::collections::HashMap;

    use super::*;
    use rtrb::{Consumer, Producer, RingBuffer};

    #[test]
    fn update_n_samples_writes_n_samples_with_constant_value() -> Result<(), InstrumentError> {
        let mut sut = DCGenerator::new("generator", 0.5);

        // we must connect the ports to a buffer to inspect the output of the instrument
        let (producer, mut consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(20);
        sut.ports()
            .output_port_mut(DCGeneratorPorts::OUT)
            .set_producer(producer)
            .map_err(|e| InstrumentError::from_port_error("generator", e))?;

        sut.update(1, 20, HashMap::new())?;

        let chunk = consumer
            .read_chunk(20)
            .expect("expected producer to have produced 20 samples");

        for (i, sample) in chunk.into_iter().enumerate() {
            assert_eq!(0.5, sample, "wrong sample at index {}", i);
        }

        Ok(())
    }
}
