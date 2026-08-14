use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct SignalSourceParameters {
    pub frequency: f32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawSourceParameters {
    pub frequency: f32,
    pub waveform: u32,
}
