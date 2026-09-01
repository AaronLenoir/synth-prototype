use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    core::{
        commands::{InstrumentCommand, ParameterId},
        instrument::instrument::Instrument,
    },
    instruments::{
        mixer::{
            channel_parameters::ChannelParameters,
            mixer::{Mixer as MixerInstrument, MixerParameters as MixerInstrumentParameters},
        },
        raw_source::{self, raw_source::RawSource},
        the_one_o_one::the_one_o_one::TheOneOhOne,
    },
};

// ********
// From here, there is code that must be extended for each new Instrument
// ********

/// Parameters for the RawSource instrument available in the config
/// - frequency: in Hz (float)
/// - waveform: 1 = sine, 2 = sawtooth (integer), 3 = square
/// - fm_depth: how much frequency modulation is applied (0.0 - 1.0), default 1.0
#[derive(Debug, Deserialize, PartialEq)]
pub struct RawSourceParameters {
    pub frequency: f32,
    pub waveform: u32,
    #[serde(default = "RawSourceParameters::default_fm_depth")]
    pub fm_depth: f32,
}

impl RawSourceParameters {
    pub fn default_fm_depth() -> f32 {
        1.0
    }
}

/// Parameters for the Mixer instrument available in the config
/// - channels: 1 - 255
#[derive(Debug, Deserialize, PartialEq)]
pub struct MixerParameters {
    pub channels: u8,
    pub master_gain: (f32, f32),
    pub channel_parameters: Vec<MixerChannelParameters>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MixerChannelParameters {
    pub gain: f32,
    pub balance: f32,
}

/// Maps to the [[instruments]] section(s) in the toml file, each
/// instrument requires an entry here, the name property is mandatory
/// any other parameters can vary per instrument
#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum InstrumentConfig {
    Mixer {
        name: String,
        parameters: MixerParameters,
    },
    RawSource {
        name: String,
        parameters: RawSourceParameters,
    },
    TheOneOhOne {
        name: String,
    },
}

/// Instrument specific logic for InstrumentConfig
/// This code will have to be extended for each new Instrument
impl InstrumentConfig {
    /// For each instrument, this code builds an Instrument instance from the a Config
    pub fn create_instrument(config: &InstrumentConfig) -> Box<dyn Instrument> {
        match config {
            InstrumentConfig::Mixer { name, parameters } => Box::new(MixerInstrument::new(
                name,
                parameters.channels,
                parameters.master_gain,
                parameters
                    .channel_parameters
                    .iter()
                    .map(|x| ChannelParameters::new(x.gain, x.balance))
                    .collect(),
            )),
            InstrumentConfig::RawSource { name, parameters } => Box::new(RawSource::new(
                name,
                parameters.frequency,
                parameters.waveform,
                parameters.fm_depth,
            )),
            InstrumentConfig::TheOneOhOne { name } => Box::new(TheOneOhOne::new(name)),
        }
    }

    /// Returns the configured name for the Instrument
    pub fn name(&self) -> &String {
        // Note: for each instrument we need to add this line, seems redundant, should be improved or made dynamic
        // at some point
        match self {
            InstrumentConfig::Mixer { name, .. } => name,
            InstrumentConfig::RawSource { name, .. } => name,
            InstrumentConfig::TheOneOhOne { name, .. } => name,
        }
    }

    /// In the configuration the parameters are referred by a name, but internally we use a ParameterId
    /// This function can map, per instrument, the parameter name to a ParameterId. In some cases various
    /// strings map to the same parameter (for example "f" and "frequency" could both resolve to RawSourceParameters::FREQUENCY)
    fn map_parameter_id(&self, parameter_name: &str) -> Result<ParameterId, InstrumentConfigError> {
        // Note: for each instrument we need to know how to map the parameter to the appropriate ParemeterId
        match self {
            InstrumentConfig::Mixer {
                name: _,
                parameters: _,
            } => {
                // The mixer (currently) has no known parameters that can be set
                if parameter_name.starts_with("GAIN.") || parameter_name.starts_with("BALANCE.") {
                    let channel_parameter =
                        MixerInstrumentParameters::map_channel_parameter(parameter_name);
                    if channel_parameter.is_some() {
                        return Ok(channel_parameter.unwrap());
                    }
                }
                match parameter_name {
                    "mgl" | "master_gain_left" => Ok(MixerInstrumentParameters::MASTER_GAIN_LEFT),
                    "mgr" | "master_gain_right" => Ok(MixerInstrumentParameters::MASTER_GAIN_LEFT),
                    _ => Err(InstrumentConfigError::UnknownParameter(
                        parameter_name.to_string(),
                    )),
                }
            }
            InstrumentConfig::RawSource {
                name: _,
                parameters: _,
            } => match parameter_name {
                "f" | "frequency" => Ok(raw_source::raw_source::RawSourceParameters::FREQUENCY),
                "w" | "waveform" => Ok(raw_source::raw_source::RawSourceParameters::WAVEFORM),
                "fm_depth" => Ok(raw_source::raw_source::RawSourceParameters::FM_DEPTH),
                _ => Err(InstrumentConfigError::UnknownParameter(
                    parameter_name.to_string(),
                )),
            },
            InstrumentConfig::TheOneOhOne { name: _ } => match parameter_name {
                // No parameters yet
                _ => Err(InstrumentConfigError::UnknownParameter(
                    parameter_name.to_string(),
                )),
            },
        }
    }
}

// ********
// Instrument specific code ends here, the code below does not have to be adjusted
// for each instrument
// ********

#[derive(Debug, PartialEq, Deserialize)]
pub enum InstrumentConfigError {
    /// The configuration contains a command that is not known
    UnknownCommand(String),
    /// Some configuration parameters for the instrument were not provided
    MissingCommandParameter(String, String),
    /// A command referred to a Parameter that is unknown by the instrument
    UnknownParameter(String),
}

/// Implementation of InstrumentConfig for generic, Instrument agnostic, logic
/// This code should not change with the introduction of a new Instrument
impl InstrumentConfig {
    pub fn build_command(
        &self,
        name: &str,
        parameters: &HashMap<String, toml::Value>,
    ) -> Result<InstrumentCommand, InstrumentConfigError> {
        if parameters.len() == 0 {
            return Ok(InstrumentCommand::Nop());
        }

        match name {
            "set" => {
                let parameter = parameters["parameter"].as_str().ok_or_else(|| {
                    InstrumentConfigError::MissingCommandParameter(
                        "set".to_string(),
                        "parameter".to_string(),
                    )
                })?;

                let value = parameters["value"].as_float().ok_or_else(|| {
                    InstrumentConfigError::MissingCommandParameter(
                        "set".to_string(),
                        "value".to_string(),
                    )
                })?;

                Ok(InstrumentCommand::Set(
                    self.map_parameter_id(parameter)?,
                    value as f32,
                ))
            }
            _ => Err(InstrumentConfigError::UnknownCommand(name.to_string())),
        }
    }
}
