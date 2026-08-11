use crate::core::rack::InstrumentId;

pub enum RackCommand {
    Instrument {
        id: InstrumentId,
        command: InstrumentCommand,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParameterId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstrumentCommand {
    Set(ParameterId, f32),
    Nop(),
}
