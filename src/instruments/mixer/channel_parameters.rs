pub struct ChannelParameters {
    pub gain: f32,
    pub balance: f32,
}

impl ChannelParameters {
    pub fn new(gain: f32, balance: f32) -> Self {
        Self { gain, balance }
    }
}
