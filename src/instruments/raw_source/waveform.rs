use crate::instruments::raw_source::wavetable_lookup::WavetableLookup;

#[derive(Clone, Copy)]
pub enum Waveform {
    None,
    Sine,
    Saw,
    Square,
}

impl Waveform {
    pub fn sample(self, phase: f32, phase_delta: f32) -> f32 {
        match self {
            Waveform::None => 0.0,
            Waveform::Sine => Waveform::sine_sample(phase),
            Waveform::Saw => {
                Waveform::saw_sample(phase)
                    - Waveform::poly_blep(phase, phase_delta)
            },
            Waveform::Square => {
                Waveform::square_sample(phase)
                    + Waveform::poly_blep(phase, phase_delta)
                    - Waveform::poly_blep((phase + 0.5) % 1.0, phase_delta)
            },
        }
    }

    fn sine_sample(phase: f32) -> f32 {
        WavetableLookup::sample(Waveform::Sine, phase)
    }

    fn saw_sample(phase: f32) -> f32 {
        2.0 * phase - 1.0
    }

    fn square_sample(phase: f32) -> f32 {
        if phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    /// Implementation based on 
    ///   - https://www.metafunction.co.uk/post/all-about-digital-oscillators-part-2-blits-bleps
    fn poly_blep(phase: f32, phase_delta: f32) -> f32 {
        if phase < phase_delta {
            // beginning of wave
            let x = phase / phase_delta;
            x + x - x * x - 1.0
        } else if phase > 1.0 - phase_delta {
            let x = (phase - 1.0) / phase_delta;
            x * x + x + x + 1.0
        } else {
            0.0
        }
    }
}