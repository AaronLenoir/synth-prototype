use std::collections::HashMap;

use crate::{
    core::{
        instrument::{
            instrument::Instrument,
            instrument_error::InstrumentError,
            instrument_info::InstrumentInfo,
            instrument_ports::{InstrumentPorts, PortId, PortResolver},
        },
        port::Ports,
    },
    instruments::{
        mixer::{
            channel_parameters::ChannelParameters,
            mixer::{Mixer, MixerOutPorts},
        },
        raw_source::{
            raw_source::{RawSource, RawSourcePorts},
            waveform::Waveform,
        },
    },
    sequencer::event::RackEvent,
};

/// Synthesizer 101, a basic synthesizer POC with two oscilators, a mixer,
/// amp and amp envelope
pub struct TheOneOhOne {
    info: InstrumentInfo,
    ports: InstrumentPorts,

    internal_instruments: HashMap<u32, Box<dyn Instrument>>,
}

// Define the Ports
pub struct TheOneOhOnePorts;

impl TheOneOhOnePorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
    pub const INTERNAL_IN_LEFT: PortId = 0;
    pub const INTERNAL_IN_RIGHT: PortId = 1;
}

pub const __OSC1: u32 = 0;
pub const __OSC2: u32 = 1;
pub const __OSC_MIXER: u32 = 2;

// Implement the PortResolver
impl PortResolver for TheOneOhOne {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(TheOneOhOnePorts::OUT_LEFT),
            "OUT_RIGHT" => Some(TheOneOhOnePorts::OUT_RIGHT),
            _ => None,
        }
    }

    fn input_port(&self, name: &str) -> Option<PortId> {
        match name {
            // currently no input ports
            "__INTERNAL_IN_LEFT" => Some(TheOneOhOnePorts::INTERNAL_IN_LEFT),
            "__INTERNAL_IN_RIGHT" => Some(TheOneOhOnePorts::INTERNAL_IN_RIGHT),
            _ => None,
        }
    }
}

impl TheOneOhOne {
    pub fn new(name: &str) -> Self {
        let mut ports = InstrumentPorts::new(2, 2);

        let mut osc1 = RawSource::new("osc1", 440.0, 1, 0.0);
        let osc2 = RawSource::new("osc1", 440.0, 0, 0.0);
        let mut mixer = Mixer::new(
            "osc_mixer",
            2,
            (1.0, 1.0),
            vec![
                ChannelParameters::new(0.5, 0.0),
                ChannelParameters::new(0.5, 0.0),
            ],
        );

        // connect osc1 outputs to mixer channel 1 inputs
        let in_left_id = mixer.input_port("IN_LEFT.1").expect("TBD");
        let in_right_id = mixer.input_port("IN_RIGHT.1").expect("TBD");

        let in_left = mixer.ports().input_port_mut(in_left_id);
        let out_left = osc1.ports().output_port_mut(RawSourcePorts::OUT_LEFT);
        Ports::connect(out_left, in_left, 48000 * 2).expect("TBD"); // TODO: would need to know the bitrate here

        let in_right = mixer.ports().input_port_mut(in_right_id);
        let out_right = osc1.ports().output_port_mut(RawSourcePorts::OUT_RIGHT);
        Ports::connect(out_right, in_right, 48000 * 2).expect("TBD"); // TODO: would need to know the bitrate here

        // Connect mixer output to our internal input ports
        let internal_in_left = ports.input_port_mut(TheOneOhOnePorts::INTERNAL_IN_LEFT);
        let mixer_out_left = mixer.ports().output_port_mut(MixerOutPorts::OUT_LEFT);
        Ports::connect(mixer_out_left, internal_in_left, 48000 * 2).expect("TBD");

        let internal_in_right = ports.input_port_mut(TheOneOhOnePorts::INTERNAL_IN_RIGHT);
        let mixer_out_right = mixer.ports().output_port_mut(MixerOutPorts::OUT_RIGHT);
        Ports::connect(mixer_out_right, internal_in_right, 48000 * 2).expect("TBD");

        let mut internal_instruments: HashMap<u32, Box<dyn Instrument>> = HashMap::new();
        internal_instruments.insert(__OSC1, Box::new(osc1));
        internal_instruments.insert(__OSC_MIXER, Box::new(mixer));

        Self {
            info: InstrumentInfo::new(name),
            ports: ports,
            internal_instruments: internal_instruments,
        }
    }
}

impl Instrument for TheOneOhOne {
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
        events: &HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        for id in [__OSC1, __OSC_MIXER] {
            let instrument = self
                .internal_instruments
                .get_mut(&id)
                .expect("internal instrument missing?");
            instrument.update(time_window, sample_count, events)?;
        }

        for sample_offset in 0..sample_count {
            self.handle_events_at_sample(sample_offset, events);

            let in_left = self
                .ports()
                .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_LEFT);
            let in_right = self
                .ports()
                .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_RIGHT);

            for port in [TheOneOhOnePorts::OUT_LEFT, TheOneOhOnePorts::OUT_RIGHT] {
                let sample = match port {
                    TheOneOhOnePorts::OUT_LEFT => {
                        let name = self.info.name().to_owned();
                        let in_left = self
                            .ports()
                            .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_LEFT);
                        in_left
                            .read_if_connected()
                            .map_err(|e| InstrumentError::from_port_error(&name, e))?
                    }
                    TheOneOhOnePorts::OUT_RIGHT => {
                        let name = self.info.name().to_owned();
                        let in_right = self
                            .ports()
                            .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_RIGHT);
                        in_right
                            .read_if_connected()
                            .map_err(|e| InstrumentError::from_port_error(&name, e))?
                    }
                    _ => 0.0,
                };
                let name = self.info.name().to_owned();
                let output = self.ports.output_port_mut(port);

                output
                    .write_if_connected(sample)
                    .map_err(|e| InstrumentError::from_port_error(&name, e))?;
            }
        }

        Ok(())
    }
}
