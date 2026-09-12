use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Default, PartialEq, Debug)]
pub struct PatternConfig {
    #[serde(default)]
    pub period: f32,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub events: Vec<HashMap<String, toml::Value>>,
}
