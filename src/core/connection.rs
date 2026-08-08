use crate::core::instrument_ports::PortId;

#[derive(Debug)]
pub struct EndPoint {
    pub instrument_name: String,
    pub port: PortId,
}

#[derive(Debug)]
pub struct Connection {
    pub source: EndPoint,
    pub target: EndPoint,
}
