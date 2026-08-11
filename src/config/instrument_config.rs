use std::collections::HashMap;

use serde::Deserialize;

use crate::{config::signal_source_parameters::{self}, core::commands::{InstrumentCommand, ParameterId}, instruments::signal_source::SignalSourceParameters};

#[derive(Debug, PartialEq, Deserialize)]
pub enum InstrumentConfigError {
    UnknownCommand(String),
    MissingCommandParameter(String, String),
    UnknownParameter(String),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum InstrumentConfig {
    Mixer {
        name: String,
    },
    SignalSource {
        name: String,
        parameters: signal_source_parameters::SignalSourceParameters,
    },
}

impl InstrumentConfig {
    pub fn name(&self) -> &String {
        // Note: for each instrument we need to add this line, seems redundant, should be improved at some point
        match self {
            InstrumentConfig::Mixer { name } => name,
            InstrumentConfig::SignalSource { name, .. } => name,
        }
    }

    fn map_parameter_id(&self, parameter_name: &str) -> Result<ParameterId, InstrumentConfigError> {
        // Note: for each instrument we need to know how to map the parameter to the appropriate ParemeterId
        match self {
            InstrumentConfig::Mixer { name: _ } => {
                // The mixer (currently) has no known parameters
                Err(InstrumentConfigError::UnknownParameter(parameter_name.to_string()))
            },
            InstrumentConfig::SignalSource { name: _, parameters: _ } => {
                match parameter_name {
                    "f" => Ok(SignalSourceParameters::FREQUENCY),
                    "frequency" => Ok(SignalSourceParameters::FREQUENCY),
                    _ => Err(InstrumentConfigError::UnknownParameter(parameter_name.to_string()))
                }
            }
        }
    }

    pub fn build_command(&self, name: &str, parameters: &HashMap<String, toml::Value>) -> Result<InstrumentCommand, InstrumentConfigError> {
        if parameters.len() == 0 {
            return Ok(InstrumentCommand::Nop());
        }

        match name {
            "set" => {
                let parameter = parameters["parameter"]
                    .as_str()
                    .ok_or_else(|| InstrumentConfigError::MissingCommandParameter("set".to_string(), "parameter".to_string()))?;

                let value = parameters["value"]
                    .as_float()
                    .ok_or_else(|| InstrumentConfigError::MissingCommandParameter("set".to_string(), "value".to_string()))?;

                Ok(InstrumentCommand::Set(self.map_parameter_id(parameter)?, value as f32))
            },
            _ => Err(InstrumentConfigError::UnknownCommand(name.to_string()))
        }
    }
}