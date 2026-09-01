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
        utils::smooth_value::SmoothValue,
    },
    instruments::mixer::channel_parameters::{self, ChannelParameters},
    sequencer::{event::RackEvent, sample_offset},
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

pub struct MixerParameters;

impl MixerParameters {
    pub const MASTER_GAIN_LEFT: ParameterId = ParameterId(0);
    pub const MASTER_GAIN_RIGHT: ParameterId = ParameterId(1);
    const PARAMETER_OFFSET: u32 = 1000;
    const BALANCE_PARAMETER_OFFSET: u32 = 2000;

    pub fn map_channel_parameter(parameter_name: &str) -> Option<ParameterId> {
        if parameter_name.starts_with("GAIN.") {
            if let Some(channel) = parameter_name
                .strip_prefix("GAIN.")
                .and_then(|n| n.parse::<u32>().ok())
            {
                return Some(ParameterId(Self::PARAMETER_OFFSET + (channel - 1)));
            } else {
                return None;
            }
        }

        if parameter_name.starts_with("BALANCE.") {
            if let Some(channel) = parameter_name
                .strip_prefix("BALANCE.")
                .and_then(|n| n.parse::<u32>().ok())
            {
                return Some(ParameterId(Self::BALANCE_PARAMETER_OFFSET + (channel - 1)));
            } else {
                return None;
            }
        }

        None
    }

    fn channel_index(value: u32) -> Option<usize> {
        if value >= Self::PARAMETER_OFFSET && value < Self::BALANCE_PARAMETER_OFFSET {
            Some((value - Self::PARAMETER_OFFSET) as usize)
        } else if value >= Self::BALANCE_PARAMETER_OFFSET {
            Some((value - Self::BALANCE_PARAMETER_OFFSET) as usize)
        } else {
            None
        }
    }
}

/// Preliminary implementation of a 16 channel Mixer, to be extended / refined
pub struct Mixer {
    info: InstrumentInfo,
    ports: InstrumentPorts,
    channels: u8,
    // left and right master gain
    master_gain: (SmoothValue, SmoothValue),
    channel_parameters: Vec<ChannelParameters>,
}

impl Mixer {
    pub fn new(
        name: &str,
        channels: u8,
        master_gain: (f32, f32),
        channel_parameters: Vec<ChannelParameters>,
    ) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(channels * 2, 2),
            channels: channels,
            master_gain: (
                SmoothValue::new(master_gain.0),
                SmoothValue::new(master_gain.1),
            ),
            channel_parameters: channel_parameters,
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
        events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        let name = self.info.name().to_owned();
        for sample_offset in 0..sample_count {
            self.handle_events_at_sample(sample_offset, &events);

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

                    let parameters = &mut self.channel_parameters[channel as usize];
                    let gain = parameters.gain.value();
                    let balance = parameters.balance.value();
                    match side {
                        Side::Left => left += (sample * gain) * (1.0 - ((balance + 1.0) / 2.0)),
                        Side::Right => right += (sample * gain) * ((balance + 1.0) / 2.0),
                    }
                }
            }

            self.ports
                .output_port_mut(MixerOutPorts::OUT_LEFT)
                .write_if_connected(left * self.master_gain.0.value())
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;

            self.ports
                .output_port_mut(MixerOutPorts::OUT_RIGHT)
                .write_if_connected(right * self.master_gain.1.value())
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        }

        Ok(())
    }

    fn handle_command(&mut self, command: crate::core::commands::InstrumentCommand) {
        match command {
            InstrumentCommand::Set(MixerParameters::MASTER_GAIN_LEFT, x) => {
                self.master_gain.0.set(x);
            }
            InstrumentCommand::Set(MixerParameters::MASTER_GAIN_RIGHT, x) => {
                self.master_gain.1.set(x);
            }
            InstrumentCommand::Set(ParameterId(value), x) => {
                let channel_index = MixerParameters::channel_index(value);
                if channel_index.is_none()
                    || channel_index.unwrap() as usize >= self.channel_parameters.len()
                {
                    // Do nothing, in purpose
                    return;
                }
                if value > MixerParameters::BALANCE_PARAMETER_OFFSET {
                    self.channel_parameters[channel_index.unwrap()]
                        .balance
                        .set(x);
                } else {
                    self.channel_parameters[channel_index.unwrap()].gain.set(x);
                }
            }
            _ => {
                // do nothing, on purpose
            }
        }
    }
}
