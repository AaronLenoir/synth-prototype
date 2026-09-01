/// Numerical value that gradually moves to its target value
pub struct SmoothValue {
    value: f32,
    target: f32,
    factor: f32,
}

impl SmoothValue {
    const TRESHOLD: f32 = 0.001;
    const DEFAULT_FACTOR: f32 = 0.01; // Default 1% per tick

    pub fn new(value: f32) -> Self {
        Self {
            value,
            target: value,
            factor: SmoothValue::DEFAULT_FACTOR,
        }
    }

    pub fn with_factor(value: f32, factor: f32) -> Self {
        Self {
            value,
            target: value,
            factor: factor,
        }
    }

    pub fn set(&mut self, target: f32) {
        self.target = target;
    }

    pub fn value(&mut self) -> f32 {
        let diff = self.target - self.value;

        if diff.abs() <= SmoothValue::TRESHOLD {
            self.value = self.target;
        } else {
            self.value += diff * self.factor;
        }

        self.value
    }
}
