use std::sync::mpsc::SendError;

use crate::core::commands::RackCommand;

#[derive(Debug)]
pub enum SequencerError {
    SendError(SendError<RackCommand>),
}
