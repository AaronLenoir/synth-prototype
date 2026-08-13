use serde::Deserialize;

use crate::config::sequencer::{clip_config::ClipConfig, meter_config::MeterConfig};

#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct SequencerConfig {
    pub tempo: u32,
    pub meter: MeterConfig,
    #[serde(default)]
    pub clips: Vec<ClipConfig>,
}
