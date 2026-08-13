use rtrb::{Consumer, Producer, RingBuffer};

use crate::core::instrument::{instrument::Instrument, instrument_error::InstrumentError, instrument_info::InstrumentInfo, instrument_ports::{InstrumentPorts, PortId, PortResolver}};

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
    ConsumerAlreadyTaken,
}

pub struct AudioOut {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    channels: u8,
    bitrate: u32,
    out_producer: Producer<f32>,
    out_consumer: Option<Consumer<f32>>,
}

impl AudioOut {
    pub fn new(name: &str, channels: u8, bitrate: u32) -> Self {
        let buffer_size = usize::from(channels) * (bitrate as usize) * 10; // 10 s max buffer
        let (producer, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(buffer_size);
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(2, 0),
            channels,
            bitrate,
            out_producer: producer,
            out_consumer: Some(consumer),
        }
    }

    pub fn take_consumer(&mut self) -> Result<Consumer<f32>, AudioOutError> {
        self.out_consumer
            .take()
            .ok_or(AudioOutError::ConsumerAlreadyTaken)
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
        let producer = &mut self.out_producer;

        if self.channels == 2 {
            for _ in 0..sample_count {
                let left = self
                    .ports
                    .input_port_mut(AudioOutPorts::IN_LEFT)
                    .read()
                    .unwrap_or(0.0);
                let right = self
                    .ports
                    .input_port_mut(AudioOutPorts::IN_RIGHT)
                    .read()
                    .unwrap_or(0.0);
                // error not ideal - we need an InstrumentError
                producer
                    .push(left)
                    .map_err(|x| InstrumentError::GeneralError(x.to_string()))?;
                producer
                    .push(right)
                    .map_err(|x| InstrumentError::GeneralError(x.to_string()))?;
            }

            return Ok(());
        }

        if self.channels == 1 {
            for _ in 0..sample_count {
                let left = self
                    .ports
                    .input_port_mut(AudioOutPorts::IN_LEFT)
                    .read()
                    .unwrap_or(0.0);
                let right = self
                    .ports
                    .input_port_mut(AudioOutPorts::IN_RIGHT)
                    .read()
                    .unwrap_or(0.0);
                // error not ideal - we need an InstrumentError
                producer
                    .push(left + right)
                    .map_err(|x| InstrumentError::GeneralError(x.to_string()))?;
            }

            return Ok(());
        }

        Err(InstrumentError::GeneralError(format!(
            "only 1 and 2 channels are supported, received: {}",
            self.channels
        )))
    }
}
