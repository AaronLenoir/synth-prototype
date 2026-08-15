use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    core::{
        commands::{InstrumentCommand, ParameterId},
        instrument::instrument::Instrument,
    },
    instruments::{
        self,
        mixer::Mixer,
        raw_source::{self, raw_source::RawSource},
        signal_source::SignalSource,
    },
};

// The following code has to be extended for each new Instrument

#[derive(Debug, Deserialize, PartialEq)]
pub struct SignalSourceParameters {
    pub frequency: f32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawSourceParameters {
    pub frequency: f32,
    pub waveform: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum InstrumentConfig {
    Mixer {
        name: String,
    },
    SignalSource {
        name: String,
        parameters: SignalSourceParameters,
    },
    RawSource {
        name: String,
        parameters: RawSourceParameters,
    },
}

impl InstrumentConfig {
    pub fn create_instrument(config: &InstrumentConfig) -> Box<dyn Instrument> {
        match config {
            InstrumentConfig::SignalSource { name, parameters } => {
                Box::new(SignalSource::new(name, parameters.frequency))
            }
            InstrumentConfig::Mixer { name } => Box::new(Mixer::new(name)),
            InstrumentConfig::RawSource { name, parameters } => Box::new(RawSource::new(
                name,
                parameters.frequency,
                parameters.waveform,
            )),
        }
    }

    pub fn name(&self) -> &String {
        // Note: for each instrument we need to add this line, seems redundant, should be improved at some point
        match self {
            InstrumentConfig::Mixer { name } => name,
            InstrumentConfig::SignalSource { name, .. } => name,
            InstrumentConfig::RawSource { name, .. } => name,
        }
    }

    fn map_parameter_id(&self, parameter_name: &str) -> Result<ParameterId, InstrumentConfigError> {
        // Note: for each instrument we need to know how to map the parameter to the appropriate ParemeterId
        match self {
            InstrumentConfig::Mixer { name: _ } => {
                // The mixer (currently) has no known parameters
                Err(InstrumentConfigError::UnknownParameter(
                    parameter_name.to_string(),
                ))
            }
            InstrumentConfig::SignalSource {
                name: _,
                parameters: _,
            } => match parameter_name {
                "f" => Ok(instruments::signal_source::SignalSourceParameters::FREQUENCY),
                "frequency" => Ok(instruments::signal_source::SignalSourceParameters::FREQUENCY),
                _ => Err(InstrumentConfigError::UnknownParameter(
                    parameter_name.to_string(),
                )),
            },
            InstrumentConfig::RawSource {
                name: _,
                parameters: _,
            } => match parameter_name {
                "f" => Ok(raw_source::raw_source::RawSourceParameters::FREQUENCY),
                "frequency" => Ok(raw_source::raw_source::RawSourceParameters::FREQUENCY),
                "w" => Ok(raw_source::raw_source::RawSourceParameters::WAVEFORM),
                "waveform" => Ok(raw_source::raw_source::RawSourceParameters::WAVEFORM),
                _ => Err(InstrumentConfigError::UnknownParameter(
                    parameter_name.to_string(),
                )),
            },
        }
    }
}

// Below here, there's no instrument specific code

#[derive(Debug, PartialEq, Deserialize)]
pub enum InstrumentConfigError {
    UnknownCommand(String),
    MissingCommandParameter(String, String),
    UnknownParameter(String),
}

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
