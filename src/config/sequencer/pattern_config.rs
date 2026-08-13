use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct PatternConfig {
    pub period: f32,
    pub command: String,
    pub events: Vec<HashMap<String, toml::Value>>,
}
