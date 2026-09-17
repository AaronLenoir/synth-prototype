use crate::core::utils::{envelope::Envelope, smooth_value::SmoothValue};

pub struct Amplifier {
    amp_envelope: Envelope,
    amp: SmoothValue,
}

impl Amplifier {
    pub fn new() -> Self {
        Self {
            amp_envelope: Envelope::new(0, 1_000_000_000, 0.5, 5_000_000_000),
            amp: SmoothValue::new(0.0),
        }
    }

    pub fn update(&mut self, time_window: u64) {
        self.amp_envelope.step(time_window);
        self.amp.set(self.amp_envelope.volume());
    }

    pub fn note_on(&mut self) {
        self.amp_envelope.start();
    }

    pub fn note_off(&mut self) {
        self.amp_envelope.release();
    }

    pub fn amplify(&mut self, sample: f32) -> f32 {
        sample * self.amp.value()
    }
}
