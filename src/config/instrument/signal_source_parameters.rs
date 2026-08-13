use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct SignalSourceParameters {
    pub frequency: f32,
}
