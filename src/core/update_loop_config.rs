pub struct UpdateLoopConfig {
    pub step_in_nanos: u128,
    pub accumulator_cap_in_nanos: u128,
    pub fps_cap_in_ms: u64,
}

impl Default for UpdateLoopConfig {
    fn default() -> Self {
        Self {
            step_in_nanos: 16_000_000,
            accumulator_cap_in_nanos: 1000000000,
            fps_cap_in_ms: 16,
        }
    }
}
