use serde::Deserialize;

use crate::config::signal_source_parameters::SignalSourceParameters;

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
}
