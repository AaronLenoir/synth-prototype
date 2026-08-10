use std::sync::mpsc::Sender;

use crate::{
    config::{clip_config::ClipConfig, config::Config, config_error::ConfigError, instrument_config::InstrumentConfigError, pattern_config::PatternConfig, rack_builder::RackBuilderError}, core::{commands::RackCommand, rack::{Rack, RackError}}, sequencer::{clip::Clip, meter::Meter, pattern::Pattern, sequencer::Sequencer, timeline_range::TimelineRange},
};

#[derive(Debug)]
pub enum SequencerBuilderError {
    Unknown,
    RackError(RackError),
    InstrumentConfigError(InstrumentConfigError),
    ConfigError(ConfigError),
}

pub struct SequencerBuilder {}

impl SequencerBuilder {
    pub fn from_config(
        command_sender: Sender<RackCommand>,
        config: &Config,
        rack: &Rack,
    ) -> Result<Sequencer, SequencerBuilderError> {
        let mut sequencer = Sequencer::new(
            config.sequencer.tempo,
            Meter {
                numerator: config.sequencer.meter.numerator,
                denominator: config.sequencer.meter.denominator,
            },
            command_sender,
        );

        Self::create_clips(&mut sequencer, config, rack);

        Ok(sequencer)
    }

    fn create_clips(sequencer: &mut Sequencer, config: &Config, rack: &Rack) {
        for clip in config.sequencer.clips.iter() {
            Self::create_clip(sequencer, config, clip, rack);
        }
    }

    fn create_clip(sequencer: &mut Sequencer, config: &Config, clip_config: &ClipConfig, rack: &Rack) -> Result<(), SequencerBuilderError> {
        let range = TimelineRange {
            start: clip_config.start.into_timeline_position(config.sequencer.meter),
            end: clip_config.end.into_timeline_position(config.sequencer.meter),
        };

        let target = rack.instrument_id(&clip_config.target).map_err(|e| SequencerBuilderError::RackError(e))?;

        let pattern = Self::create_pattern(config, clip_config)?;

        sequencer.add_clip(Clip::new(range, target, pattern));

        Ok(())
    }

    fn create_pattern(config: &Config, clip_config: &ClipConfig) -> Result<Pattern, SequencerBuilderError> {
        // get instrument type
        let instrument = config.instrument_config_by_name(&clip_config.target).map_err(|e| SequencerBuilderError::ConfigError(e))?;

        let mut commands = Vec::new();

        for event in &clip_config.pattern.events {
            let command = instrument.build_command(&clip_config.pattern.command.as_str(), event)
                .map_err(|e| SequencerBuilderError::InstrumentConfigError(e))?;
            commands.push(command);
        }

        Ok(Pattern {
            period: clip_config.pattern.period,
            commands: commands,
        })
    }
}

// TODO: Unit Test the Builder