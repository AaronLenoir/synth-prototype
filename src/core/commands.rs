use crate::rack::rack::InstrumentId;

pub enum RackCommand {
    Instrument {
        id: InstrumentId,
        command: InstrumentCommand,
    },
}

impl RackCommand {
    pub fn instrument_id(&self) -> InstrumentId {
        match self {
            RackCommand::Instrument { id, .. } => *id,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParameterId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstrumentCommand {
    Set(ParameterId, f32),
    Nop(),
}
