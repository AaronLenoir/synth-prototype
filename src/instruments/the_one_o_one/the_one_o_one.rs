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
    instruments::{
        mixer::{
            channel_parameters::ChannelParameters,
            mixer::{Mixer, MixerOutPorts},
        },
        raw_source::raw_source::{RawSource, RawSourceParameters, RawSourcePorts},
    },
    rack::rack::Rack,
    sequencer::event::RackEvent,
};

/// Synthesizer 101, a basic synthesizer POC with two oscilators, a mixer,
/// amp and amp envelope
pub struct TheOneOhOne {
    info: InstrumentInfo,
    ports: InstrumentPorts,

    internal_rack: Option<Rack>,
}

// Define the Ports
pub struct TheOneOhOnePorts;

impl TheOneOhOnePorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
    pub const INTERNAL_IN_LEFT: PortId = 0;
    pub const INTERNAL_IN_RIGHT: PortId = 1;
}

// Define the Parameters
pub struct TheOneOhOneParameters;

impl TheOneOhOneParameters {
    pub const FREQUENCY: ParameterId = ParameterId(1);
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
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(2, 2),
            internal_rack: None,
        }
    }

    fn get_sample_from_input(&mut self, port: PortId) -> Result<f32, InstrumentError> {
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

        Ok(sample)
    }

    fn build_rack(&self, sample_rate: u32) -> Rack {
        // Create an empty rack
        let mut rack = Rack::without_audio_out(sample_rate);

        // Create the necessary internal instruments
        let osc1 = RawSource::new("osc1", 440.0, 1, 0.0);
        let osc2 = RawSource::new("osc2", 441.0, 1, 0.0);
        let mixer = Mixer::new(
            "osc_mixer",
            2,
            (1.0, 1.0),
            vec![
                ChannelParameters::new(0.5, 0.0),
                ChannelParameters::new(0.5, 0.0),
            ],
        );

        // now add the instruments BEFORE we can connect them (the rack needs to know them)
        rack.add_instrument(Box::new(osc1))
            .expect("cannot add instrument");
        rack.add_instrument(Box::new(osc2))
            .expect("cannot add instrument");
        rack.add_instrument(Box::new(mixer))
            .expect("cannot add instrument");

        rack
    }
}

impl Instrument for TheOneOhOne {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn initialize(&mut self, sample_rate: u32) -> Result<(), InstrumentError> {
        // Create an empty rack
        let mut rack = self.build_rack(sample_rate);

        let mut mixer = rack.instrument("osc_mixer");

        let mixer = mixer.as_mut().expect("missing osc_mixer instrument");

        // We ask the PortId's of the mixer before we add the mixer to the rack
        // because at that time we lose ownership
        let (mixer_1_left, mixer_1_right, mixer_2_left, mixer_2_right) = (
            mixer
                .input_port("IN_LEFT.1")
                .expect("mixer missing port IN_LEFT.1"),
            mixer
                .input_port("IN_RIGHT.1")
                .expect("mixer missing port IN_RIGHT.1"),
            mixer
                .input_port("IN_LEFT.2")
                .expect("mixer missing port IN_LEFT.2"),
            mixer
                .input_port("IN_RIGHT.2")
                .expect("mixer missing port IN_RIGHT.2"),
        );

        // Connect mixer output to our internal input ports
        let internal_in_left = self
            .ports
            .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_LEFT);

        rack.connect_external("osc_mixer", MixerOutPorts::OUT_LEFT, internal_in_left)
            .map_err(|e| {
                InstrumentError::GeneralError(format!(
                    "Could not connect synth to internal mixer: {:?}",
                    e
                ))
            })?;

        let internal_in_right = self
            .ports
            .input_port_mut(TheOneOhOnePorts::INTERNAL_IN_RIGHT);

        rack.connect_external("osc_mixer", MixerOutPorts::OUT_RIGHT, internal_in_right)
            .map_err(|e| {
                InstrumentError::GeneralError(format!(
                    "Could not connect synth to internal mixer: {:?}",
                    e
                ))
            })?;

        rack.connect_direct("osc1", RawSourcePorts::OUT_LEFT, "osc_mixer", mixer_1_left)
            .map_err(|e| {
                InstrumentError::GeneralError(format!("Could not connect osc1 to mixer: {:?}", e))
            })?;

        rack.connect_direct(
            "osc1",
            RawSourcePorts::OUT_RIGHT,
            "osc_mixer",
            mixer_1_right,
        )
        .map_err(|e| {
            InstrumentError::GeneralError(format!("Could not connect osc1 to mixer: {:?}", e))
        })?;

        rack.connect_direct("osc2", RawSourcePorts::OUT_LEFT, "osc_mixer", mixer_2_left)
            .map_err(|e| {
                InstrumentError::GeneralError(format!("Could not connect osc2 to mixer: {:?}", e))
            })?;

        rack.connect_direct(
            "osc2",
            RawSourcePorts::OUT_RIGHT,
            "osc_mixer",
            mixer_2_right,
        )
        .map_err(|e| {
            InstrumentError::GeneralError(format!("Could not connect osc2 to mixer: {:?}", e))
        })?;

        self.internal_rack = Some(rack);

        Ok(())
    }

    fn update(
        &mut self,
        time_window: u128,
        sample_count: u32,
        events: &HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        let one_sample_window = time_window / sample_count as u128;

        for sample_offset in 0..sample_count {
            // we can only update the rack one sample at a time because we need to properly
            // handle events that may alter the instrument parameters at any moment in the window?
            self.internal_rack
                .as_mut()
                .expect("missing internal rack")
                .update(one_sample_window, 1, vec![])
                .expect("update failed");

            self.handle_events_at_sample(sample_offset, events);

            for port in [TheOneOhOnePorts::OUT_LEFT, TheOneOhOnePorts::OUT_RIGHT] {
                let sample = self.get_sample_from_input(port)?;
                let name = self.info.name().to_owned();
                let output = self.ports.output_port_mut(port);

                output
                    .write_if_connected(sample)
                    .map_err(|e| InstrumentError::from_port_error(&name, e))?;
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, command: crate::core::commands::InstrumentCommand) {
        match command {
            crate::core::commands::InstrumentCommand::Set(
                TheOneOhOneParameters::FREQUENCY,
                value,
            ) => {
                if self.internal_rack.is_none() {
                    return;
                }
                self.internal_rack
                    .as_mut()
                    .expect("")
                    .instrument("osc1")
                    .expect("osc1 missing")
                    .handle_command(InstrumentCommand::Set(
                        RawSourceParameters::FREQUENCY,
                        value,
                    ));
                self.internal_rack
                    .as_mut()
                    .expect("")
                    .instrument("osc2")
                    .expect("osc1 missing")
                    .handle_command(InstrumentCommand::Set(
                        RawSourceParameters::FREQUENCY,
                        value * 0.80,
                    ));
            }
            crate::core::commands::InstrumentCommand::Note(note, velocity) => {
                if velocity.0 > 0.0 {
                    // Note on
                    // Here, set the frequency according to the note
                    //   maybe: let f = self.oscilator1.map_note_to_frequency(note)
                    //     (the oscilator config object can keep that logic, including settings like transpose I dunno)
                    // initialise the envelope
                    //   maybe: self.envelope.reset()
                    self.internal_rack
                        .as_mut()
                        .expect("")
                        .instrument("osc1")
                        .expect("osc1 missing")
                        .handle_command(InstrumentCommand::Set(
                            RawSourceParameters::FREQUENCY,
                            800.0,
                        ));
                } else {
                    // Here set the envelop to release
                    //   maybe: self.envelope.release()
                    self.internal_rack
                        .as_mut()
                        .expect("")
                        .instrument("osc1")
                        .expect("osc1 missing")
                        .handle_command(InstrumentCommand::Set(
                            RawSourceParameters::FREQUENCY,
                            1.0,
                        ));
                }
            }
            _ => {}
        }
    }
}
