use crate::core::{commands::ParameterId, instrument::{instrument::Instrument, instrument_error::InstrumentError, instrument_info::InstrumentInfo, instrument_ports::{InstrumentPorts, PortId, PortResolver}}};

pub struct DelayPorts;

impl DelayPorts {
    pub const IN_LEFT: PortId = 0;
    pub const IN_RIGHT: PortId = 1;

    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
}

pub struct DelayParameters;

impl DelayParameters {
    pub const DELAY: ParameterId = ParameterId(0);
    pub const DECAY: ParameterId = ParameterId(1);
}

pub struct Delay {
    info: InstrumentInfo,
    ports: InstrumentPorts,

    delay: f32,
    decay: f32,
}

impl Delay {
    pub fn new(name: &str, delay: f32, decay: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(2, 2),
            delay: delay,
            decay: decay,
        }
    }

    pub fn process_sample(&mut self, in_port: PortId, out_port: PortId) -> Result<(), InstrumentError>  {
        let in_port = self.ports.input_port_mut(in_port);

        let input_sample = in_port.read_if_connected().unwrap_or(0.0);

        let out_port = self.ports.output_port_mut(out_port);

        out_port.write_if_connected(
            input_sample
        ).map_err(|e| InstrumentError::from_port_error(&self.info.name().to_owned(), e))?;

        Ok(())
    }
}

impl Instrument for Delay {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(
        &mut self,
        _time_window: u128,
        sample_count: u32,
        _events: &std::collections::HashMap<u32, Vec<&crate::sequencer::event::RackEvent>>,
    ) -> Result<(), InstrumentError>
    {
        for _ in 0..sample_count { 
            self.process_sample(DelayPorts::IN_LEFT, DelayPorts::OUT_LEFT)?;
            self.process_sample(DelayPorts::IN_RIGHT, DelayPorts::OUT_RIGHT)?;
        }

        Ok(())
    }
}

impl PortResolver for Delay {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(DelayPorts::OUT_LEFT),
            "OUT_RIGHT" => Some(DelayPorts::OUT_RIGHT),
            _ => None,
        }
    }
    fn input_port(&self, name: &str) -> Option<PortId> {
        match name {
            "IN_LEFT" => Some(DelayPorts::IN_LEFT),
            "IN_RIGHT" => Some(DelayPorts::IN_RIGHT),
            _ => None,
        }
    }
}
