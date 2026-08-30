use crate::core::utils::smooth_value::SmoothValue;

pub struct ChannelParameters {
    pub gain: SmoothValue,
    pub balance: SmoothValue,
}

impl ChannelParameters {
    pub fn new(gain: f32, balance: f32) -> Self {
        Self {
            gain: SmoothValue::new(gain),
            balance: SmoothValue::new(balance),
        }
    }
}
