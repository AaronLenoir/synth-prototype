use std::{os::unix::process::parent_id, sync::mpsc::Receiver};

use crate::{
    config::{
        config::Config,
        instrument::instrument_config::InstrumentConfig,
        rack::connection_config::{ConnectionConfig, EndPointConfig},
    },
    core::{commands::RackCommand, instrument::instrument_ports::PortId},
    instruments::{mixer::Mixer, raw_source::raw_source::RawSource, signal_source::SignalSource},
    rack::{
        connection::{Connection, EndPoint},
        rack::{Rack, RackError},
    },
};

#[derive(Debug)]
pub enum RackBuilderError {
    UnknownPort(String, String),
    RackError(RackError),
}

pub struct RackBuilder {}

impl RackBuilder {
    pub fn from_config(
        command_receiver: Receiver<RackCommand>,
        config: &Config,
    ) -> Result<Rack, RackBuilderError> {
        let mut rack = Rack::new(command_receiver, config.rack.bitrate);

        Self::create_instruments(&mut rack, config)?;
        Self::create_connections(&mut rack, config)?;

        Ok(rack)
    }

    fn create_instruments(rack: &mut Rack, config: &Config) -> Result<(), RackBuilderError> {
        for instrument in config.instruments.iter() {
            Self::create_instrument(rack, instrument)?;
        }

        Ok(())
    }

    fn create_instrument(
        rack: &mut Rack,
        config: &InstrumentConfig,
    ) -> Result<(), RackBuilderError> {
        let result = match config {
            InstrumentConfig::SignalSource { name, parameters } => {
                let signal_source = SignalSource::new(name, parameters.frequency);
                rack.add_instrument(Box::new(signal_source))
            }
            InstrumentConfig::Mixer { name } => {
                let mixer = Mixer::new(name);
                rack.add_instrument(Box::new(mixer))
            }
            InstrumentConfig::RawSource { name, parameters } => {
                let raw_source = RawSource::new(name, parameters.frequency, parameters.waveform);
                rack.add_instrument(Box::new(raw_source))
            }
        };
        result.map_err(|e| RackBuilderError::RackError(e))?;

        Ok(())
    }

    fn create_connections(rack: &mut Rack, config: &Config) -> Result<(), RackBuilderError> {
        for connection in config.connections.iter() {
            Self::create_connection(rack, connection)?;
        }

        Ok(())
    }

    fn create_connection(
        rack: &mut Rack,
        config: &ConnectionConfig,
    ) -> Result<(), RackBuilderError> {
        let connection = Connection {
            source: EndPoint {
                instrument_name: config.source.instrument.to_string(),
                port: Self::parse_output_port(rack, &config.source)?,
            },
            target: EndPoint {
                instrument_name: config.target.instrument.to_string(),
                port: Self::parse_input_port(rack, &config.target)?,
            },
        };

        rack.connect(connection)
            .map_err(|e| RackBuilderError::RackError(e))?;

        Ok(())
    }

    fn parse_input_port(
        rack: &mut Rack,
        config: &EndPointConfig,
    ) -> Result<PortId, RackBuilderError> {
        let instrument = rack
            .instrument(&config.instrument)
            .map_err(|e| RackBuilderError::RackError(e))?;
        match instrument.input_port(&config.port) {
            Some(port) => Ok(port),
            _ => Err(RackBuilderError::UnknownPort(
                config.instrument.to_string(),
                config.port.to_string(),
            )),
        }
    }

    fn parse_output_port(
        rack: &mut Rack,
        config: &EndPointConfig,
    ) -> Result<PortId, RackBuilderError> {
        let instrument = rack
            .instrument(&config.instrument)
            .map_err(|e| RackBuilderError::RackError(e))?;
        match instrument.output_port(&config.port) {
            Some(port) => Ok(port),
            _ => Err(RackBuilderError::UnknownPort(
                config.instrument.to_string(),
                config.port.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod rack_builder_tests {
    use std::sync::mpsc::{self, Sender};

    use crate::{
        config::rack::rack_config::RackConfig,
        instruments::mixer::{MixerInPorts, MixerOutPorts},
    };

    use super::*;

    fn get_receiver() -> Receiver<RackCommand> {
        let (_, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();
        rx
    }

    #[test]
    fn from_config_builds_rack_with_bitrate() {
        let config = Config::new(RackConfig { bitrate: 48000 }, None);

        let rack = RackBuilder::from_config(get_receiver(), &config).unwrap();

        assert_eq!(rack.bitrate, 48000);
    }

    #[test]
    fn from_config_builds_rack_with_instruments() {
        let mut config = Config::new(RackConfig { bitrate: 48000 }, None);
        config.instruments = vec![InstrumentConfig::Mixer {
            name: "mixer".to_string(),
        }];

        let rack = RackBuilder::from_config(get_receiver(), &config).unwrap();

        let result = rack.instrument_id(&"mixer".to_string());

        assert!(result.is_ok(), "Unexpected error: {:?}", result);
    }

    #[test]
    fn from_config_builds_connection() {
        let mut config = Config::new(RackConfig { bitrate: 48000 }, None);
        config.instruments = vec![
            InstrumentConfig::Mixer {
                name: "mixer1".to_string(),
            },
            InstrumentConfig::Mixer {
                name: "mixer2".to_string(),
            },
        ];

        config.connections = vec![ConnectionConfig {
            source: EndPointConfig {
                instrument: "mixer1".to_string(),
                port: "OUT_LEFT".to_string(),
            },
            target: EndPointConfig {
                instrument: "mixer2".to_string(),
                port: "IN_LEFT_01".to_string(),
            },
        }];

        let mut rack = RackBuilder::from_config(get_receiver(), &config).unwrap();

        // to check if the connection was made we try to make it again and should get an error
        let test_connection = Connection {
            source: EndPoint {
                instrument_name: "mixer1".to_string(),
                port: MixerOutPorts::OUT_LEFT,
            },
            target: EndPoint {
                instrument_name: "mixer2".to_string(),
                port: MixerInPorts::IN_LEFT_01,
            },
        };

        rack.connect(test_connection)
            .expect_err("connection should have already existed");
    }
}
