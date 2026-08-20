use crate::sequencer::{duration::Duration, timeline_position::TimelinePosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleOffset {
    pub value: u32,
}

impl SampleOffset {
    pub fn new(window: Duration, sample_count: u32, offset: TimelinePosition) -> Self {
        let samples_per_beat = sample_count as f32 / window.value;

        Self {
            value: (offset.value * samples_per_beat) as u32,
        }
    }

    pub fn from_value(value: u32) -> Self {
        Self { value }
    }
}

#[cfg(test)]
mod clip_tests {

    use super::*;

    #[test]
    fn calculates_sample_offset() {
        let cases = [
            (
                Duration::new(10.0),
                20 as u32,
                TimelinePosition::new(0.0),
                0,
            ),
            (
                Duration::new(10.0),
                20 as u32,
                TimelinePosition::new(10.0),
                20,
            ),
            (
                Duration::new(10.0),
                20 as u32,
                TimelinePosition::new(5.0),
                10,
            ),
            (
                Duration::new(10.0),
                756 as u32,
                TimelinePosition::new(5.0),
                378,
            ),
            (
                Duration::new(10.0),
                756 as u32,
                TimelinePosition::new(2.5),
                189,
            ),
            (
                Duration::new(200.0),
                987000 as u32,
                TimelinePosition::new(123.0),
                607005,
            ),
        ];

        for (window, sample_count, offset, expected_value) in cases {
            let sut = SampleOffset::new(window, sample_count, offset);
            assert_eq!(sut.value, expected_value);
        }
    }
}
