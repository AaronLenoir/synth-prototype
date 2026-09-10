use crate::rack::rack::InstrumentId;

/// The Rack receives a RackCommand which wraps an InstrumentCommand and the ID
/// of one of the Rack's instruments
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

/// Used to identify a Parameter in the Set command
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParameterId(pub u32);

/// For a note event, specifies the musical note (0 - 127 matching MIDI?)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NoteNumber(pub u32);

/// For a note event, specifies the velocity 0.0 - 1.0 (velocity = 0.0 == Note Off)
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Velocity(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstrumentCommand {
    /// Command that sets a parameter to a specific value (float)
    /// Available parameters differ per instrument
    Set(ParameterId, f32),
    Note(NoteNumber, Velocity),
    /// Command that is expected to do nothing
    Nop(),
}
