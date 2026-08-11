use std::sync::mpsc::Sender;

use crate::{
    config::{config::Config, sequencer_builder::{SequencerBuilder, SequencerBuilderError}}, core::{commands::RackCommand, rack::Rack}, sequencer::{
        clip::Clip, meter::Meter, sequencer_error::SequencerError,
        timeline_position::TimelinePosition, timeline_range::TimelineRange,
    },
};

pub struct Sequencer {
    /// In BPM (beats per minute)
    pub tempo: u32,

    // The meter (metre) of the score
    pub meter: Meter,

    pub clips: Vec<Clip>,

    sender: Sender<RackCommand>,

    playing: bool,

    position: TimelinePosition,
}

impl Sequencer {
    pub fn new(tempo: u32, meter: Meter, sender: Sender<RackCommand>) -> Self {
        Self {
            tempo: tempo,
            meter: meter,
            clips: vec![],
            sender: sender,
            playing: false,
            position: 0.0,
        }
    }

    pub fn from_config(
        command_sender: Sender<RackCommand>,
        config: &Config,
        rack: &Rack,
    ) -> Result<Sequencer, SequencerBuilderError> {
        let sequencer = SequencerBuilder::from_config(command_sender, &config, rack)?;

        Ok(sequencer)
    }

    pub fn update(&mut self, time_window: u128, sample_count: u32) -> Result<(), SequencerError> {
        if !self.playing {
            return Ok(());
        }

        let start_position = self.position;
        // How many beats in a time_window?
        // time_window in nanoseconds (time_windows / 1_000_000_000) = seconds
        // bpm in minutes (bpm / 60) = beats per second
        let step = (self.tempo as f32 / 60.0) * (time_window as f32 / 1_000_000_000.0);

        self.position = self.position + step;

        let mut commands: Vec<RackCommand> = vec![];
        for clip in self.clips.iter() {
            let clip_commands = clip.commands_between(TimelineRange {
                start: start_position,
                end: self.position,
            });

            clip_commands.iter().for_each(|c| {
                commands.push(RackCommand::Instrument {
                    id: clip.target,
                    command: *c,
                })
            });
        }

        for command in commands {
            self.sender
                .send(command)
                .map_err(|e| SequencerError::SendError(e))?;
        }

        Ok(())
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
        core::{
            commands::{InstrumentCommand, ParameterId, RackCommand},
            rack::InstrumentId,
        },
        sequencer::{
            clip::Clip, meter::Meter, pattern::Pattern, sequencer::Sequencer,
            sequencer_error::SequencerError, timeline_range::TimelineRange,
        },
    };

    fn get_sut(sender: Sender<RackCommand>) -> Sequencer {
        let mut sequencer = Sequencer::new(
            120,
            Meter {
                numerator: 4,
                denominator: 4,
            },
            sender,
        );

        sequencer.add_clip(get_clip_single_command(TimelineRange {
            start: 0.0,
            end: 10.0,
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
                period: 1.0,
                commands: vec![InstrumentCommand::Set(ParameterId(1), 100.0)],
            },
        )
    }

    #[test]
    fn position_moves_according_to_time_windows() {
        let (tx, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();

        let mut sut = get_sut(tx);

        sut.play();
        sut.update(1_000_000_000, 1000).expect("unexpected error");

        // One second progression at 120 bpm = 2 beats
        assert_eq!(2.0, sut.position);
    }

    #[test]
    fn command_sent_at_interval() {
        let (tx, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();

        let mut sut = get_sut(tx);

        sut.play();
        sut.update(1_000_000_000, 1000).expect("unexpected error");

        // After 1 second, our pattern should've sent two commands, we should see them on the channel
        let mut commands: Vec<RackCommand> = vec![];
        commands.push(rx.recv().expect("unexpected error"));
        commands.push(rx.recv().expect("unexpected error"));

        assert_eq!(2, commands.len());
    }
}
