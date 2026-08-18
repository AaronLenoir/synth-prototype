use std::collections::HashMap;

use rtrb::{Consumer, Producer, RingBuffer};

use crate::core::instrument::{
    instrument::Instrument,
    instrument_error::InstrumentError,
    instrument_info::InstrumentInfo,
    instrument_ports::{InstrumentPorts, PortId, PortResolver},
};

pub struct AudioOutPorts;

impl AudioOutPorts {
    pub const IN_LEFT: PortId = 0;
    pub const IN_RIGHT: PortId = 1;
}

impl PortResolver for AudioOut {
    fn output_port(&self, _name: &str) -> Option<PortId> {
        // we have no output ports
        None
    }
    fn input_port(&self, name: &str) -> Option<PortId> {
        match name {
            "IN_LEFT" => Some(AudioOutPorts::IN_LEFT),
            "IN_RIGHT" => Some(AudioOutPorts::IN_RIGHT),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AudioOutError {
    PortAlreadyConnected(PortId),
}

pub struct AudioOut {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    producers: HashMap<PortId, Producer<f32>>,
}

impl AudioOut {
    pub fn new(name: &str) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(2, 0),
            producers: HashMap::new(),
        }
    }

    pub fn connect_producer(
        &mut self,
        producer: Producer<f32>,
        port: PortId,
    ) -> Result<(), AudioOutError> {
        if self.producers.contains_key(&port) {
            Err(AudioOutError::PortAlreadyConnected(port))
        } else {
            self.producers.insert(port, producer);
            Ok(())
        }
    }
}

impl Instrument for AudioOut {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), InstrumentError> {
        // read from the input ports, write to the internal buffer which will be read by
        // cpal's "move"

        for port in [AudioOutPorts::IN_LEFT, AudioOutPorts::IN_RIGHT] {
            if !self.producers.contains_key(&port) {
                continue;
            }
            let producer_item = self.producers.get_mut(&port);
            if producer_item.is_none() {
                continue;
            }
            let producer = producer_item.unwrap();
            for _ in 0..sample_count {
                let sample = self
                    .ports
                    .input_port_mut(port)
                    .read_if_connected()
                    .unwrap_or(0.0);

                producer
                    .push(sample)
                    .map_err(|x| InstrumentError::GeneralError(x.to_string()))?;
            }
        }

        Ok(())
    }
}
