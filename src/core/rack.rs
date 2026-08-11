use std::{collections::HashMap, sync::mpsc::Receiver};

use rtrb::{Consumer, Producer, RingBuffer};
use slotmap::{SlotMap, new_key_type};

use crate::{
    config::{
        config::Config,
        rack_builder::{RackBuilder, RackBuilderError},
    },
    core::{
        audio_device::AudioDevice, commands::RackCommand, connection::Connection,
        connection_order::ConnectionOrder, instrument::Instrument,
        instrument_error::InstrumentError, port::PortError,
    },
    instruments::audio_out::AudioOut,
};

#[derive(Debug)]
pub enum RackError {
    InstrumentNameAlreadyExists,
    InstrumentError(InstrumentError),
    PortError(PortError),
    InstrumentNotFound(String),
    CannotConnectInstrumentToItself(String),
    UnknownConnection(Connection),
    InvalidInstrumentPair(InstrumentId, InstrumentId),
}

#[derive(Debug)]
pub enum LoadRackError {
    RackBuilder(RackBuilderError),
}

new_key_type! {
    pub struct InstrumentId;
}

const AUDIO_OUT_CHANNELS: u8 = 2;

pub struct Rack {
    instruments: SlotMap<InstrumentId, Box<dyn Instrument + 'static>>,
    instrument_id_map: HashMap<String, InstrumentId>,
    connections: Vec<Connection>,
    connection_order: ConnectionOrder,
    audio_device: AudioDevice,
    command_receiver: Receiver<RackCommand>,
    pub bitrate: u32,
    pub audio_out_name: String,
}

impl Rack {
    pub fn new(command_receiver: Receiver<RackCommand>, bitrate: u32) -> Self {
        let audio_out_name = "__AUDIO_OUT";

        let mut audio_out = AudioOut::new(audio_out_name, AUDIO_OUT_CHANNELS, bitrate);

        let mut audio_device = AudioDevice::new();
        audio_device.connect_consumer(audio_out.take_consumer().expect("error"));

        let mut instrument_id_map: HashMap<String, InstrumentId> = HashMap::new();
        let mut instruments: SlotMap<InstrumentId, Box<dyn Instrument + 'static>> =
            SlotMap::with_key();
        let id = instruments.insert(Box::new(audio_out));
        instrument_id_map.insert(audio_out_name.to_string(), id);

        Self {
            instruments: instruments,
            instrument_id_map: instrument_id_map,
            connections: Vec::new(),
            connection_order: ConnectionOrder::new(&vec![]),
            audio_device: audio_device,
            audio_out_name: audio_out_name.to_string(),
            command_receiver: command_receiver,
            bitrate: bitrate,
        }
    }

    pub fn from_config(
        command_receiver: Receiver<RackCommand>,
        config: &Config,
    ) -> Result<Rack, RackBuilderError> {
        let rack = RackBuilder::from_config(command_receiver, &config)?;

        Ok(rack)
    }

    pub fn play(&mut self) {
        self.audio_device.play();
    }

    // add instrument
    pub fn add_instrument(&mut self, instrument: Box<dyn Instrument>) -> Result<(), RackError> {
        let instrument_name = instrument.info().name().to_string();

        if self.instrument_id_map.get(&instrument_name).is_some() {
            return Err(RackError::InstrumentNameAlreadyExists);
        }

        let id = self.instruments.insert(instrument);
        self.instrument_id_map
            .insert(instrument_name.to_string(), id);

        Ok(())
    }

    pub fn connect(&mut self, connection: Connection) -> Result<(), RackError> {
        let buffer_size = (self.bitrate as usize) * 2; // 2 seconds

        let (source, target) = self.instrument_pair(
            &connection.source.instrument_name,
            &connection.target.instrument_name,
        )?;

        let output = source.ports().output_port_mut(connection.source.port);
        let input = target.ports().input_port_mut(connection.target.port);

        // create a buffer and connect producer and consumer
        let (producer, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(buffer_size);
        output
            .set_producer(producer)
            .map_err(RackError::PortError)?;
        input.set_consumer(consumer).map_err(RackError::PortError)?;

        // keep track of connections
        self.connections.push(connection);

        // re-order the connections
        self.connection_order = ConnectionOrder::new(&self.connections);

        Ok(())
    }

    pub fn disconnect(&mut self, connection: Connection) -> Result<(), RackError> {
        if !self.connections.iter().any(|x| {
            x.source.instrument_name == connection.source.instrument_name
                && x.source.port == connection.source.port
                && x.target.instrument_name == connection.target.instrument_name
                && x.target.port == connection.target.port
        }) {
            return Err(RackError::UnknownConnection(connection));
        }

        let (source, target) = self.instrument_pair(
            &connection.source.instrument_name,
            &connection.target.instrument_name,
        )?;

        let output = source.ports().output_port_mut(connection.source.port);
        let input = target.ports().input_port_mut(connection.target.port);

        output.remove_producer().map_err(RackError::PortError)?;
        input.remove_consumer().map_err(RackError::PortError)?;

        // re-order the connections
        self.connection_order = ConnectionOrder::new(&self.connections);

        Ok(())
    }

    // update method
    pub fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), RackError> {
        for instrument_name in &self.connection_order.instruments {
            let instrument_id = self
                .instrument_id_map
                .get(instrument_name)
                .copied()
                .ok_or_else(|| RackError::InstrumentNotFound(instrument_name.to_string()))?;

            let instrument = self
                .instruments
                .get_mut(instrument_id)
                .expect("Instrument should exist");

            instrument
                .update(time_window, sample_count)
                .map_err(|e| RackError::InstrumentError(e))?;
        }

        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                RackCommand::Instrument { id, command } => {
                    let instrument = self
                        .instruments
                        .get_mut(id)
                        .expect("Instrument should exist");
                    instrument.handle_command(command);
                }
            }
        }

        Ok(())
    }

    pub fn instrument(&mut self, name: &str) -> Result<&mut dyn Instrument, RackError> {
        let instrument_id = self
            .instrument_id_map
            .get(name)
            .ok_or(RackError::InstrumentNotFound(name.to_string()))?;
        match self.instruments.get_mut(*instrument_id) {
            None => Err(RackError::InstrumentNotFound(name.to_owned())),
            Some(i) => Ok(i.as_mut()),
        }
    }

    fn instrument_pair(
        &mut self,
        source_name: &str,
        target_name: &str,
    ) -> Result<(&mut dyn Instrument, &mut dyn Instrument), RackError> {
        if source_name == target_name {
            return Err(RackError::CannotConnectInstrumentToItself(
                source_name.to_owned(),
            ));
        }

        let source_id = self
            .instrument_id_map
            .get(source_name)
            .copied()
            .ok_or(RackError::InstrumentNotFound(source_name.to_owned()))?;
        let target_id = self
            .instrument_id_map
            .get(target_name)
            .copied()
            .ok_or(RackError::InstrumentNotFound(target_name.to_owned()))?;

        let [source, target] = self
            .instruments
            .get_disjoint_mut([source_id, target_id])
            .ok_or_else(|| RackError::InvalidInstrumentPair(source_id, target_id))?;

        Ok((source.as_mut(), target.as_mut()))
    }

    pub fn instrument_id(&self, name: &str) -> Result<InstrumentId, RackError> {
        let id = self
            .instrument_id_map
            .get(name)
            .copied()
            .ok_or(RackError::InstrumentNotFound(name.to_owned()))?;

        Ok(id)
    }
}

#[cfg(test)]
mod rack_tests {
    use std::sync::mpsc::{self, Sender};

    use crate::{
        core::connection::EndPoint,
        instruments::{
            audio_out::{AudioOut, AudioOutPorts},
            dc_generator::{DCGenerator, DCGeneratorPorts},
        },
    };

    use super::*;

    fn get_rack() -> Rack {
        let (_, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();
        let rack = Rack::new(rx, 48000);
        rack
    }

    #[test]
    fn cannot_add_instrument_if_name_in_use() {
        let instrument1 = DCGenerator::new("instrumentA", 0.5);
        let instrument2 = DCGenerator::new("instrumentA", 0.5);

        let mut rack = get_rack();

        rack.add_instrument(Box::new(instrument1))
            .expect("add_instrument failed unexpectedly");

        assert!(matches!(
            rack.add_instrument(Box::new(instrument2)),
            Err(RackError::InstrumentNameAlreadyExists)
        ));
    }

    // Tests if two instruments can be added to the rack, connected and if the output
    // of one instrument can be found on the input of the other
    #[test]
    fn connect_connects_two_instrument_ports() {
        let instrument1 = DCGenerator::new("instrumentA", 0.5);

        let mut rack = get_rack();

        rack.add_instrument(Box::new(instrument1))
            .expect("add_instrument failed unexpectedly");

        let connection = Connection {
            source: EndPoint {
                instrument_name: "instrumentA".to_string(),
                port: DCGeneratorPorts::OUT,
            },
            target: EndPoint {
                instrument_name: rack.audio_out_name.clone(),
                port: AudioOutPorts::IN_LEFT,
            },
        };

        rack.connect(connection)
            .expect("connect failed unexpectedly");

        let source = rack
            .instrument("instrumentA")
            .expect("could not find instrument");

        source
            .ports()
            .output_port_mut(DCGeneratorPorts::OUT)
            .write_if_connected(123.0)
            .expect("write to instrument1 failed unexpectedly");

        let target = rack
            .instrument(rack.audio_out_name.clone().as_str())
            .expect("could not find instrument");

        let result_from_input = target
            .ports()
            .input_port_mut(AudioOutPorts::IN_LEFT)
            .read()
            .expect("read failed unexpectedly");

        assert_eq!(result_from_input, 123.0)
    }

    // Tests if two instruments can be added to the rack, connected and if the output
    // of one instrument can be found on the input of the other
    #[test]
    fn disconnect_will_disconnect_port() {
        let instrument1 = DCGenerator::new("instrumentA", 0.5);
        let instrument2 = AudioOut::new("instrumentB", 1, 48000);

        let mut rack = get_rack();

        rack.add_instrument(Box::new(instrument1))
            .expect("add_instrument failed unexpectedly");
        rack.add_instrument(Box::new(instrument2))
            .expect("add_instrument failed unexpectedly");

        rack.connect(Connection {
            source: EndPoint {
                instrument_name: "instrumentA".to_string(),
                port: DCGeneratorPorts::OUT,
            },
            target: EndPoint {
                instrument_name: "instrumentB".to_string(),
                port: AudioOutPorts::IN_LEFT,
            },
        })
        .expect("connect failed unexpectedly");
        rack.disconnect(Connection {
            source: EndPoint {
                instrument_name: "instrumentA".to_string(),
                port: DCGeneratorPorts::OUT,
            },
            target: EndPoint {
                instrument_name: "instrumentB".to_string(),
                port: AudioOutPorts::IN_LEFT,
            },
        })
        .expect("disconnect failed unexpectedly");

        let source = rack
            .instrument("instrumentA")
            .expect("could not find instrument");

        let is_connected = source
            .ports()
            .output_port_mut(DCGeneratorPorts::OUT)
            .is_connected();

        assert_eq!(is_connected, false);

        let target = rack
            .instrument("instrumentB")
            .expect("could not find instrument");

        let read_error = target
            .ports()
            .input_port_mut(AudioOutPorts::IN_LEFT)
            .read()
            .expect_err("write should have failed");

        assert!(matches!(read_error, PortError::NotConnected));
    }
}
