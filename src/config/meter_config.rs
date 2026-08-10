use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug, Default, Copy, Clone)]
pub struct MeterConfig {
    pub numerator: i32,
    pub denominator: i32,
}
