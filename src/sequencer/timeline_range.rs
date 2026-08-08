use crate::sequencer::timeline_position::TimelinePosition;

pub struct TimelineRange {
    pub start: TimelinePosition,
    pub end: TimelinePosition,
}

impl TimelineRange {
    pub fn is_in_range(&self, position: TimelinePosition) -> bool {
        position >= self.start && position < self.end
    }

    pub fn overlaps(&self, range: &TimelineRange) -> bool {
        self.start < range.end && range.start < self.end
    }
}
