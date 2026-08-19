use crate::sequencer::timeline_position::TimelinePosition;

/// A duration in the sequence expressed in beats.
///
/// A value of:
/// - `1.0` represents one beat,
/// - `0.5` represents half a beat,
/// - `4.0` represents four beats.
pub type Duration = TimelinePosition;
