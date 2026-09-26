pub struct DelayBuffer {
    delay: usize,
    decay: f32,
    buffer: Vec<f32>,

    input_ptr: usize,
    output_ptr: usize,

    last_output: f32,
}

impl DelayBuffer {
    /// Create a new delay buffer
    ///   - delay: how much of delay before the signal is echo's (expressed in samples)
    ///   - decay: how much decay after each iteration of feedback (0.0 - 1.0)
    ///   - buffer_size: the maximum delay (expressed in samples)
    pub fn new(delay: usize, decay: f32, buffer_size: usize) -> Self {
        Self {
            delay: delay,
            decay: decay,
            buffer: vec![0.0; buffer_size],

            input_ptr: delay,
            output_ptr: 0,

            last_output: 0.0,
        }
    }

    /// Sample comes in to the delay buffer
    pub fn push(&mut self, sample: f32) {
        self.buffer[self.input_ptr] = sample + (self.last_output * self.decay);

        self.input_ptr = self.increment_ptr(self.input_ptr);
    }

    /// Sample is taken out of the buffer output - last output is recorded for 
    /// feedback loop
    pub fn pop(&mut self) -> f32 {
        self.last_output = self.buffer[self.output_ptr];

        self.output_ptr = self.increment_ptr(self.output_ptr);

        self.last_output
    }

    fn increment_ptr(&self, ptr: usize) -> usize {
        if ptr + 1 >= self.buffer.len() {
            0
        } else {
            ptr + 1
        }
    }
}

#[cfg(test)]
mod delay_buffer_tests {

    use super::*;

    #[test]
    fn first_pop_is_zero() {
        let mut sut = DelayBuffer::new(1, 0.0, 2);

        let sample = sut.pop();

        assert_eq!(sample, 0.0);
    }

    #[test]
    fn pop_after_delay_returns_input() {
        // delay is one sample
        let mut sut = DelayBuffer::new(1, 0.0, 2);

        sut.push(0.5);
        assert_eq!(sut.pop(), 0.0);

        sut.push(0.4);
        assert_eq!(sut.pop(), 0.5); // we expect the second sample to be our previous input
    }

    #[test]
    fn pop_after_delay_mixes_decayed_feedback() {
        // delay is one sample
        let mut sut = DelayBuffer::new(1, 0.8, 20);

        sut.push(0.5);
        assert_eq!(sut.pop(), 0.0); 
        sut.push(0.5);
        assert_eq!(sut.pop(), 0.5); // here we want to see the first delayed sample
        sut.push(0.5); 
        assert_eq!(sut.pop(), 0.5); // here we expect the second pushed sample
        sut.push(0.5); 
        assert_eq!(sut.pop(), 0.5 + 0.5 * 0.8); // here we expect the third sample + the feedback

    }
}