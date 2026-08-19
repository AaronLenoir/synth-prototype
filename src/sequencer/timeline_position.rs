use std::ops::AddAssign;

use crate::sequencer::duration::Duration;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimelinePosition {
    pub value: f32,
}

impl TimelinePosition {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl AddAssign<Duration> for TimelinePosition {
    fn add_assign(&mut self, duration: Duration) {
        self.value += duration.value;
    }
}