use crate::{core::{commands::ParameterId, instrument::{instrument::Instrument, instrument_error::InstrumentError, instrument_info::InstrumentInfo, instrument_ports::{InstrumentPorts, PortId, PortResolver}}}, instruments::delay::delay_buffer::DelayBuffer};

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

    sample_rate: u32,

    delay: f32,
    decay: f32,

    buffers: Vec<DelayBuffer>,
}

impl Delay {
    pub fn new(name: &str, delay: f32, decay: f32) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(2, 2),
            delay: delay,
            decay: decay,

            buffers: vec![],

            sample_rate: 1,
        }
    }

    pub fn process_sample(&mut self, in_port: PortId, out_port: PortId) -> Result<(), InstrumentError>  {
        let buffer_index = match in_port {
            DelayPorts::IN_LEFT => 0,
            DelayPorts::IN_RIGHT => 1,
            _ => return Err(InstrumentError::GeneralError("no buffer exists".to_string())),
        };
        let in_port = self.ports.input_port_mut(in_port);
        let input_sample = in_port.read_if_connected().unwrap_or(0.0);

        self.buffers[buffer_index].push(input_sample);

        let out_port = self.ports.output_port_mut(out_port);
        out_port.write_if_connected(
            self.buffers[buffer_index].pop()
        ).map_err(|e| InstrumentError::from_port_error(&self.info.name().to_owned(), e))?;

        Ok(())
    }

    /// Based on the delay (in seconds) and the sample_rate calculates how many samples to delay
    fn get_delay_in_samples(&self) -> usize {
        (self.sample_rate as f32 * self.delay) as usize
    }
}

impl Instrument for Delay {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn initialize(&mut self, sample_rate: u32) -> Result<(), InstrumentError> {
        self.sample_rate = sample_rate;

        let buffer_size = (sample_rate * 10) as usize; // max 10 seconds

        self.buffers.push(DelayBuffer::new(self.get_delay_in_samples(), self.decay, buffer_size));
        self.buffers.push(DelayBuffer::new(self.get_delay_in_samples(), self.decay, buffer_size));

        Ok(())
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
