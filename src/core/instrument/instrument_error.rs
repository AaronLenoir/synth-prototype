use crate::core::port::PortError;

#[derive(Debug)]
pub enum InstrumentError {
    GeneralError(String),
    PortError(String, PortError),
}

impl InstrumentError {
    pub fn from_port_error(name: &str, err: PortError) -> Self {
        InstrumentError::PortError(name.to_string(), err)
    }
}
