use crate::core::port::{InputPort, OutputPort};

pub type PortId = usize;

pub trait PortResolver {
    fn output_port(&self, name: &str) -> Option<PortId>;
    fn input_port(&self, name: &str) -> Option<PortId>;
}

pub struct InstrumentPorts {
    input_ports: Vec<InputPort>,
    output_ports: Vec<OutputPort>,
}

impl InstrumentPorts {
    pub fn new(input_ports: u8, output_ports: u8) -> Self {
        InstrumentPorts {
            input_ports: (0..input_ports).map(|_| InputPort::new()).collect(),
            output_ports: (0..output_ports).map(|_| OutputPort::new()).collect(),
        }
    }

    pub fn input_port_mut(&mut self, id: PortId) -> &mut InputPort {
        &mut self.input_ports[id]
    }

    pub fn output_port_mut(&mut self, id: PortId) -> &mut OutputPort {
        &mut self.output_ports[id]
    }
}
