use std::sync::mpsc::Sender;

use crate::{
    config::{
        builder::sequencer_builder::{SequencerBuilder, SequencerBuilderError},
        config::Config,
    },
    core::commands::RackCommand,
    rack::rack::Rack,
    sequencer::{
        clip::Clip, duration::Duration, event::RackEvent, meter::Meter,
        sample_offset::SampleOffset, sequencer_error::SequencerError,
        timeline_position::TimelinePosition, timeline_range::TimelineRange,
    },
};

pub struct Sequencer {
    /// In BPM (beats per minute)
    pub tempo: u32,

    // The meter (metre) of the score
    pub meter: Meter,

    pub clips: Vec<Clip>,

    playing: bool,

    position: TimelinePosition,
}

impl Sequencer {
    pub fn new(tempo: u32, meter: Meter) -> Self {
        Self {
            tempo: tempo,
            meter: meter,
            clips: vec![],
            playing: false,
            position: TimelinePosition::new(0.0),
        }
    }

    pub fn from_config(config: &Config, rack: &Rack) -> Result<Sequencer, SequencerBuilderError> {
        let sequencer = SequencerBuilder::from_config(&config, rack)?;

        Ok(sequencer)
    }

    pub fn step(
        &mut self,
        time_window: u128,
        sample_count: u32,
    ) -> Result<Vec<RackEvent>, SequencerError> {
        if !self.playing {
            return Ok(vec![]);
        }

        let mut result: Vec<RackEvent> = vec![];

        let start_position = self.position;
        // How many beats in a time_window?
        // time_window in nanoseconds (time_windows / 1_000_000_000) = seconds
        // bpm in minutes (bpm / 60) = beats per second
        let step =
            Duration::new((self.tempo as f32 / 60.0) * (time_window as f32 / 1_000_000_000.0));

        self.position += step;

        for clip in self.clips.iter() {
            let clip_commands = clip.commands_between(TimelineRange {
                start: start_position,
                end: self.position,
            });

            clip_commands.iter().for_each(|c| {
                let offset = TimelinePosition::new(c.position.value - start_position.value);
                result.push(RackEvent::new(
                    SampleOffset::new(step, sample_count, offset),
                    RackCommand::Instrument {
                        id: clip.target,
                        command: c.command,
                    },
                ));
            });
        }

        Ok(result)
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip)
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }
}

#[cfg(test)]
mod clip_tests {
    use std::sync::mpsc::{self, Receiver, Sender};

    use slotmap::SlotMap;

    use crate::{
        core::commands::{InstrumentCommand, ParameterId, RackCommand},
        rack::rack::InstrumentId,
        sequencer::{
            clip::Clip, duration::Duration, meter::Meter, pattern::Pattern, sequencer::Sequencer,
            timeline_position::TimelinePosition, timeline_range::TimelineRange,
        },
    };

    fn get_sut() -> Sequencer {
        let mut sequencer = Sequencer::new(
            120,
            Meter {
                numerator: 4,
                denominator: 4,
            },
        );

        sequencer.add_clip(get_clip_single_command(TimelineRange {
            start: TimelinePosition::new(0.0),
            end: TimelinePosition::new(10.0),
        }));

        sequencer
    }

    fn get_dummy_instrument_id() -> InstrumentId {
        let mut dummy_instruments: SlotMap<InstrumentId, bool> = SlotMap::with_key();
        let dummy_instrument_id = dummy_instruments.insert(true);

        return dummy_instrument_id;
    }

    fn get_clip_single_command(range: TimelineRange) -> Clip {
        Clip::new(
            range,
            get_dummy_instrument_id(),
            Pattern {
                period: Duration::new(1.0),
                commands: vec![InstrumentCommand::Set(ParameterId(1), 100.0)],
            },
        )
    }

    #[test]
    fn position_moves_according_to_time_windows() {
        let mut sut = get_sut();

        sut.play();
        let _ = sut.step(1_000_000_000, 1000).expect("unexpected error");

        // One second progression at 120 bpm = 2 beats
        assert_eq!(2.0, sut.position.value);
    }

    #[test]
    fn events_at_interval() {
        let mut sut = get_sut();

        sut.play();
        let events = sut.step(1_000_000_000, 1000).expect("unexpected error");

        // After 1 second, our pattern should've produced two events, we should see them on the channel
        assert_eq!(2, events.len());
    }
}
