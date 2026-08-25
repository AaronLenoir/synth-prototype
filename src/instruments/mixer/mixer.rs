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

#[derive(Copy, Clone)]
enum Side {
    Left,
    Right,
}

// Define the Ports
pub struct MixerOutPorts;

impl MixerOutPorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
}

/// Preliminary implementation of a 16 channel Mixer, to be extended / refined
pub struct Mixer {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    channels: u8,
    master_gain: f32,
}

impl Mixer {
    pub fn new(name: &str, channels: u8, master_gain: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(channels * 2, 2),
            channels: channels,
            master_gain: master_gain,
        }
    }

    fn port_id(&self, channel: u8, side: Side) -> Result<PortId, InstrumentError> {
        if channel >= self.channels {
            Err(InstrumentError::GeneralError(
                "non-existing channel used".to_string(),
            ))
        } else {
            match side {
                Side::Left => Ok((channel * 2) as PortId),
                Side::Right => Ok((1 + channel * 2) as PortId),
            }
        }
    }
}

impl PortResolver for Mixer {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(MixerOutPorts::OUT_LEFT),
            "OUT_RIGHT" => Some(MixerOutPorts::OUT_RIGHT),
            _ => None,
        }
    }

    fn input_port(&self, name: &str) -> Option<PortId> {
        let (prefix, index) = name.split_once(".")?;
        let side = match prefix {
            "IN_LEFT" => Side::Left,
            "IN_RIGHT" => Side::Right,
            _ => return None,
        };

        let channel_index = index.parse::<u8>().ok()?.checked_sub(1)?;

        self.port_id(channel_index, side).ok()
    }
}

impl Instrument for Mixer {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(
        &mut self,
        _time_window: u128,
        sample_count: u32,
        _events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        let sample_count_as_usize = sample_count as usize;

        let name = self.info.name().to_owned();
        for _ in 0..sample_count_as_usize {
            let mut left: f32 = 0.0;
            let mut right: f32 = 0.0;

            for channel in 0..self.channels {
                for side in [Side::Left, Side::Right] {
                    let port_id = self.port_id(channel, side)?;

                    let sample = self
                        .ports
                        .input_port_mut(port_id)
                        .read_if_connected()
                        .unwrap_or(0.0);

                    match side {
                        Side::Left => left += sample,
                        Side::Right => right += sample,
                    }
                }
            }

            self.ports
                .output_port_mut(MixerOutPorts::OUT_LEFT)
                .write_if_connected(left * self.master_gain)
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;

            self.ports
                .output_port_mut(MixerOutPorts::OUT_RIGHT)
                .write_if_connected(right * self.master_gain)
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        }

        Ok(())
    }
}
