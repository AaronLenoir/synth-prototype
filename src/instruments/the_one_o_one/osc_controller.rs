pub struct OscController {
    frequency: f32,
    octave_shift: i32,
}

impl OscController {
    pub fn new(frequency: f32, octave_shift: i32) -> Self {
        Self {
            frequency,
            octave_shift,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_octave_shift(&mut self, octave_shift: i32) {
        self.octave_shift = octave_shift;
    }

    pub fn output_frequency(&self) -> f32 {
        if self.octave_shift == 0 {
            self.frequency
        } else if self.octave_shift > 0 {
            self.frequency * self.octave_shift as f32
        } else {
            self.frequency / self.octave_shift as f32
        }
    }
}
