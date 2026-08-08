/// The position of the sequencer within the score
pub struct MusicalPosition {
    /// Bar number in the score.
    pub bar: u32,

    /// Beat number in the score.
    pub beat: f32,

    /// Fractional position within the beat.
    pub offset: f32,
}
