use std::fs;

use crate::config::{
    config_error::{
        ConfigError::{self, TomlError},
        LoadConfigError,
    },
    instrument::instrument_config::InstrumentConfig,
    rack::{
        connection_config::{ConnectionConfig, EndPointConfigTuples},
        rack_config::RackConfig,
    },
    sequencer::sequencer_config::SequencerConfig,
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
    #[serde(default)]
    pub sequencer: SequencerConfig,
}

impl Config {
    pub fn new(rack: RackConfig, sequencer: Option<SequencerConfig>) -> Config {
        Config {
            rack: rack,
            instruments: vec![],
            connection_tuples: EndPointConfigTuples { endpoints: vec![] },
            connections: vec![],
            sequencer: sequencer.unwrap_or_default(),
        }
    }

    /// Read a toml file from the given path and deserialize it to a Config object (if possible)
    pub fn from_file(path: String) -> Result<Config, LoadConfigError> {
        let contents = fs::read_to_string(path).map_err(|e| LoadConfigError::Io(e))?;
        let config = Config::from_str(&contents).map_err(|e| LoadConfigError::Config(e))?;

        Ok(config)
    }

    /// Given a string containing text in valid toml deserializes to a Config
    pub fn from_str(s: &str) -> Result<Config, ConfigError> {
        let mut config: Config = toml::from_str(s).map_err(|e| TomlError(e))?;

        config.connections =
            ConnectionConfig::from_connection_tuples(&config.connection_tuples.endpoints);

        Ok(config)
    }

    /// From an instrument name, finds the matching InstrumentConfig
    pub fn instrument_config_by_name(&self, name: &str) -> Result<&InstrumentConfig, ConfigError> {
        let instrument_config = self.instruments.iter().find(|i| i.name() == name);
        if instrument_config.is_none() {
            Err(ConfigError::UnknownInstrumentName(name.to_string()))
        } else {
            Ok(instrument_config.unwrap())
        }
    }
}

#[cfg(test)]
mod config_tests {
    use std::collections::HashMap;

    use crate::config::{
        instrument::instrument_config::{
            MixerChannelParameters, MixerParameters, RawSourceParameters,
        },
        rack::connection_config::EndPointConfig,
        sequencer::{
            clip_config::ClipConfig, meter_config::MeterConfig, musical_position::MusicalPosition,
            pattern_config::PatternConfig,
        },
    };

    use super::*;

    #[test]
    fn from_str_parses_instrument_config() {
        let sut = Config::from_str(
            r#"
        [rack]
        
        [[instruments]]
        type="Mixer"
        name="mixer1"
        [instruments.parameters]
        channels=1
        master_gain=[1.0, 1.0]
        [[instruments.parameters.channel_parameters]]
        gain=1.0
        balance=0.0

        [[instruments]]
        type="RawSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0
        waveform=1
        "#,
        )
        .unwrap();

        assert_eq!(
            sut.instruments,
            vec![
                InstrumentConfig::Mixer {
                    name: "mixer1".to_string(),
                    parameters: MixerParameters {
                        channels: 1,
                        master_gain: (1.0, 1.0),
                        channel_parameters: vec![MixerChannelParameters {
                            gain: 1.0,
                            balance: 0.0
                        },],
                    }
                },
                InstrumentConfig::RawSource {
                    name: "generator1".to_string(),
                    parameters: RawSourceParameters {
                        frequency: 1000.0,
                        waveform: 1,
                        fm_depth: 1.0,
                    },
                },
            ]
        );
    }

    #[test]
    fn from_str_parses_connection_tuples() {
        let sut = Config::from_str(
            r#"
        [rack]
        
        [[instruments]]
        type="Mixer"
        name="mixer1"
        [instruments.parameters]
        channels=1
        master_gain=[1.0, 1.0]
        [[instruments.parameters.channel_parameters]]
        gain=1.0
        balance=0.0

        [[instruments]]
        type="RawSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0
        waveform=1

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
        
        [[instruments]]
        type="Mixer"
        name="mixer1"
        [instruments.parameters]
        channels=1
        master_gain=[1.0, 1.0]
        [[instruments.parameters.channel_parameters]]
        gain=1.0
        balance=0.0

        [[instruments]]
        type="RawSource"
        name="generator1"
        [instruments.parameters]
        frequency=1000.0
        waveform=1

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

    #[test]
    fn from_str_parses_sequencer_and_clips() {
        let sut = Config::from_str(
            r#"
        [rack]
        
        [[instruments]]
        type="Mixer"
        name="mixer1"
        [instruments.parameters]
        channels=1
        master_gain=[1.0, 1.0]
        [[instruments.parameters.channel_parameters]]
        gain=1.0
        balance=0.0

        [sequencer]
        tempo=120
        meter={ numerator = 4, denominator = 4 }

        [[sequencer.clips]]
        start={ bar = 1, beat = 1, offset = 0.0 }
        end={ bar = 2, beat = 1, offset = 0.0 } 
        target = "mixer1"

        [sequencer.clips.pattern]
        period = 1
        command = "set"

        events = [
            { parameter = "frequency", value = 500 },
            { parameter = "frequency", value = 300 },
        ]

        "#,
        )
        .unwrap();

        assert_eq!(sut.sequencer.tempo, 120);

        assert_eq!(
            sut.sequencer.meter,
            MeterConfig {
                numerator: 4,
                denominator: 4
            }
        );

        assert_eq!(sut.sequencer.clips.len(), 1);

        assert_eq!(
            sut.sequencer.clips[0],
            ClipConfig {
                start: MusicalPosition {
                    bar: 1,
                    beat: 1,
                    offset: 0.0
                },
                end: MusicalPosition {
                    bar: 2,
                    beat: 1,
                    offset: 0.0
                },
                target: "mixer1".to_string(),
                pattern: PatternConfig {
                    period: 1.0,
                    command: "set".to_string(),
                    events: vec![
                        HashMap::from([
                            (
                                "parameter".to_string(),
                                toml::Value::String("frequency".to_string())
                            ),
                            ("value".to_string(), toml::Value::Integer(500)),
                        ]),
                        HashMap::from([
                            (
                                "parameter".to_string(),
                                toml::Value::String("frequency".to_string())
                            ),
                            ("value".to_string(), toml::Value::Integer(300)),
                        ]),
                    ],
                },
            }
        );
    }
}
