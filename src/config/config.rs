use crate::config::{
    config_error::ConfigError::{self, TomlError},
    connection_config::{
        ConnectionConfig, EndPointConfig, EndPointConfigTuple, EndPointConfigTuples,
    },
    instrument_config::InstrumentConfig,
    rack_config::RackConfig,
};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub rack: RackConfig,
    #[serde(default)]
    pub instruments: Vec<InstrumentConfig>,
    #[serde(default, rename = "connections")]
    connection_tuples: EndPointConfigTuples,
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
}

impl Config {
    pub fn new(rack: RackConfig) -> Config {
        Config {
            rack: rack,
            instruments: vec![],
            connection_tuples: EndPointConfigTuples { endpoints: vec![] },
            connections: vec![],
        }
    }

    pub fn from_str(s: &str) -> Result<Config, ConfigError> {
        let mut config: Config = toml::from_str(s).map_err(|e| TomlError(e))?;

        config.connections =
            ConnectionConfig::from_connection_tuples(&config.connection_tuples.endpoints);

        Ok(config)
    }
}

#[cfg(test)]
mod input_port_tests {
    use std::collections::HashMap;

    use crate::config::{
        connection_config::EndPointConfig, signal_source_parameters::SignalSourceParameters,
    };

    use super::*;

    fn params(entries: &[(&str, toml::Value)]) -> HashMap<String, toml::Value> {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn from_str_parses_rack_bitrate() {
        let sut = Config::from_str(
            r#"
        [rack]
        bitrate = 48000
        "#,
        )
        .unwrap();

        assert_eq!(sut.rack, RackConfig { bitrate: 48_000 });
    }

    #[test]
    fn from_str_parses_instrument_config() {
        let sut = Config::from_str(
            r#"
        [rack]
        bitrate = 48000
        
        [[instruments]]
        type="Mixer"
        name="mixer1"

        [[instruments]]
        type="SignalSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0
        "#,
        )
        .unwrap();

        assert_eq!(
            sut.instruments,
            vec![
                InstrumentConfig::Mixer {
                    name: "mixer1".to_string(),
                },
                InstrumentConfig::SignalSource {
                    name: "generator1".to_string(),
                    parameters: SignalSourceParameters { frequency: 1000.0 },
                },
            ]
        );
    }

    #[test]
    fn from_str_parses_connection_tuples() {
        let sut = Config::from_str(
            r#"
        [rack]
        bitrate = 48000
        
        [[instruments]]
        type="Mixer"
        name="mixer1"

        [[instruments]]
        type="SignalSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0

        [connections]
        endpoints = [
            ["generator1", "OUT_LEFT", "mixer1", "IN_LEFT_01"]
        ]

        "#,
        )
        .unwrap();

        assert_eq!(
            sut.connection_tuples.endpoints,
            vec![(
                "generator1".to_string(),
                "OUT_LEFT".to_string(),
                "mixer1".to_string(),
                "IN_LEFT_01".to_string()
            )]
        );
    }

    #[test]
    fn from_str_parses_connection_config() {
        let sut = Config::from_str(
            r#"
        [rack]
        bitrate = 48000
        
        [[instruments]]
        type="Mixer"
        name="mixer1"

        [[instruments]]
        type="SignalSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0

        [connections]
        endpoints = [
            ["generator1", "OUT_LEFT", "mixer1", "IN_LEFT_01"]
        ]

        "#,
        )
        .unwrap();

        assert_eq!(
            sut.connections,
            vec![ConnectionConfig {
                source: EndPointConfig {
                    instrument: "generator1".to_string(),
                    port: "OUT_LEFT".to_string()
                },
                target: EndPointConfig {
                    instrument: "mixer1".to_string(),
                    port: "IN_LEFT_01".to_string()
                },
            }]
        );
    }
}
