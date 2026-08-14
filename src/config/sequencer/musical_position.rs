use serde::Deserialize;

use crate::{
    config::sequencer::meter_config::MeterConfig, sequencer::timeline_position::TimelinePosition,
};

/// The position within the score expressed in bar, beat and beat offset
#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct MusicalPosition {
    /// Bar number in the score.
    pub bar: u32,

    /// Beat number in the score.
    pub beat: u32,

    /// Fractional position within the beat.
    pub offset: f32,
}

impl MusicalPosition {
    pub fn into_timeline_position(self, meter: MeterConfig) -> TimelinePosition {
        (meter.numerator * (self.bar - 1) as i32) as f32 + (self.beat - 1) as f32 + self.offset
    }
}

#[cfg(test)]
mod musical_position_tests {

    use super::*;

    fn get_parameters(
        numerator: i32,
        denominator: i32,
        bar: u32,
        beat: u32,
        offset: f32,
        expected_beats: f32,
    ) -> (MeterConfig, MusicalPosition, TimelinePosition) {
        (
            MeterConfig {
                numerator,
                denominator,
            },
            MusicalPosition { bar, beat, offset },
            expected_beats,
        )
    }

    #[test]
    fn into_timeline_position_converts_bar_to_beats_using_meter() {
        let mut parameters: Vec<(MeterConfig, MusicalPosition, TimelinePosition)> = vec![];
        parameters.push(get_parameters(4, 4, 1, 1, 0.0, 0.0));
        parameters.push(get_parameters(3, 4, 5, 2, 0.0, 13.0));
        parameters.push(get_parameters(3, 4, 5, 2, 0.5, 13.5));

        for (meter, position, expected_beats) in parameters {
            assert_eq!(position.into_timeline_position(meter), expected_beats);
        }
    }
}
