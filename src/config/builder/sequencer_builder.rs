use std::sync::mpsc::Sender;

use crate::{
    config::{
        config::Config, config_error::ConfigError,
        instrument::instrument_config::InstrumentConfigError, sequencer::clip_config::ClipConfig,
    },
    core::commands::RackCommand,
    rack::rack::{Rack, RackError},
    sequencer::{
        clip::Clip, duration::Duration, meter::Meter, pattern::Pattern, sequencer::Sequencer,
        timeline_range::TimelineRange,
    },
};

#[derive(Debug)]
pub enum SequencerBuilderError {
    RackError(RackError),
    InstrumentConfigError(InstrumentConfigError),
    ConfigError(ConfigError),
}

pub struct SequencerBuilder {}

impl SequencerBuilder {
    pub fn from_config(config: &Config, rack: &Rack) -> Result<Sequencer, SequencerBuilderError> {
        let mut sequencer = Sequencer::new(
            config.sequencer.tempo,
            Meter {
                numerator: config.sequencer.meter.numerator,
                denominator: config.sequencer.meter.denominator,
            },
        );

        Self::create_clips(&mut sequencer, config, rack)?;

        Ok(sequencer)
    }

    fn create_clips(
        sequencer: &mut Sequencer,
        config: &Config,
        rack: &Rack,
    ) -> Result<(), SequencerBuilderError> {
        for clip in config.sequencer.clips.iter() {
            Self::create_clip(sequencer, config, clip, rack)?;
        }

        Ok(())
    }

    fn create_clip(
        sequencer: &mut Sequencer,
        config: &Config,
        clip_config: &ClipConfig,
        rack: &Rack,
    ) -> Result<(), SequencerBuilderError> {
        let range = TimelineRange {
            start: clip_config
                .start
                .into_timeline_position(config.sequencer.meter),
            end: clip_config
                .end
                .into_timeline_position(config.sequencer.meter),
        };

        let target = rack
            .instrument_id(&clip_config.target)
            .map_err(|e| SequencerBuilderError::RackError(e))?;

        let pattern = Self::create_pattern(config, clip_config)?;

        sequencer.add_clip(Clip::new(range, target, pattern));

        Ok(())
    }

    fn create_pattern(
        config: &Config,
        clip_config: &ClipConfig,
    ) -> Result<Pattern, SequencerBuilderError> {
        // get instrument type
        let instrument = config
            .instrument_config_by_name(&clip_config.target)
            .map_err(|e| SequencerBuilderError::ConfigError(e))?;

        let mut commands = Vec::new();

        for event in &clip_config.pattern.events {
            let command = instrument
                .build_command(&clip_config.pattern.command.as_str(), event)
                .map_err(|e| SequencerBuilderError::InstrumentConfigError(e))?;
            commands.push(command);
        }

        Ok(Pattern {
            period: Duration::new(clip_config.pattern.period),
            commands: commands,
        })
    }
}

#[cfg(test)]
mod sequencer_builder_tests {
    use std::sync::mpsc::{self, Receiver};

    use crate::config::builder::rack_builder::RackBuilder;

    use super::*;

    #[test]
    fn from_config_builds_sequencer_with_tempo() {
        let config = Config::from_str(
            r#"
        [rack]

        [sequencer]
        tempo=120
        meter={ numerator = 4, denominator = 4 }
        "#,
        )
        .unwrap();

        let sequencer = SequencerBuilder::from_config(&config, &Rack::new()).unwrap();

        assert_eq!(sequencer.tempo, 120);
        assert_eq!(sequencer.meter.numerator, 4);
        assert_eq!(sequencer.meter.denominator, 4);
    }

    #[test]
    fn from_config_builds_sequencer_with_clips() {
        let config = Config::from_str(
            r#"
        [rack]

        [[instruments]]
        type="RawSource"
        name="signal1"

        [instruments.parameters]
        frequency=200
        waveform=1

        [sequencer]
        tempo=120
        meter={ numerator = 4, denominator = 4 }

        [[sequencer.clips]]
        start={ bar = 1, beat = 1, offset = 0.0 }
        end={ bar = 2, beat = 1, offset = 0.0 } 
        target = "signal1"

        [sequencer.clips.pattern]
        period = 1
        command = "set"

        events = [
            { parameter = "frequency", value = 500.0 },
            { parameter = "frequency", value = 300.0 },
        ]
        "#,
        )
        .unwrap();

        let rack = RackBuilder::from_config(&config).unwrap();
        let sequencer = SequencerBuilder::from_config(&config, &rack).unwrap();

        assert_eq!(sequencer.clips.len(), 1);
    }
}
