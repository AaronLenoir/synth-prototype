use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct RackConfig {
    pub bitrate: u32,
}
