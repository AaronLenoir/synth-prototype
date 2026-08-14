use crate::{
    core::commands::InstrumentCommand,
    rack::rack::InstrumentId,
    sequencer::{
        pattern::Pattern, timeline_position::TimelinePosition, timeline_range::TimelineRange,
    },
};

pub struct Clip {
    range: TimelineRange,
    pub target: InstrumentId,
    pattern: Pattern,
}

impl Clip {
    pub fn new(range: TimelineRange, target: InstrumentId, pattern: Pattern) -> Self {
        Self {
            range,
            target,
            pattern,
        }
    }

    pub fn commands_between(&self, range: TimelineRange) -> Vec<InstrumentCommand> {
        let mut result = vec![];

        if !self.range.overlaps(&range) || self.pattern.commands.len() == 0 {
            // we are not in the current range so inactive
            return result;
        }

        let mut position: TimelinePosition = self.range.start;
        let mut pattern_index: usize = 0;

        while position < range.end && position < self.range.end {
            if range.is_in_range(position) {
                result.push(self.pattern.commands[pattern_index]);
            }

            position += self.pattern.period;
            pattern_index += 1;
            if pattern_index >= self.pattern.commands.len() {
                pattern_index = 0;
            }
        }

        return result;
    }
}

#[cfg(test)]
mod clip_tests {
    use slotmap::SlotMap;

    use crate::core::commands::ParameterId;

    use super::*;

    fn get_dummy_instrument_id() -> InstrumentId {
        let mut dummy_instruments: SlotMap<InstrumentId, bool> = SlotMap::with_key();
        let dummy_instrument_id = dummy_instruments.insert(true);

        return dummy_instrument_id;
    }

    fn get_sut_single_command(range: TimelineRange) -> Clip {
        Clip::new(
            range,
            get_dummy_instrument_id(),
            Pattern {
                period: 1.0,
                commands: vec![InstrumentCommand::Set(ParameterId(1), 100.0)],
            },
        )
    }

    fn get_sut_10_commands(range: TimelineRange, period: f32) -> Clip {
        let mut dummy_instruments: SlotMap<InstrumentId, bool> = SlotMap::with_key();
        let dummy_instrument_id = dummy_instruments.insert(true);

        Clip::new(
            range,
            dummy_instrument_id,
            Pattern {
                period: period,
                commands: vec![
                    InstrumentCommand::Set(ParameterId(1), 100.0),
                    InstrumentCommand::Set(ParameterId(1), 200.0),
                    InstrumentCommand::Set(ParameterId(1), 300.0),
                    InstrumentCommand::Set(ParameterId(1), 400.0),
                    InstrumentCommand::Set(ParameterId(1), 500.0),
                    InstrumentCommand::Set(ParameterId(1), 600.0),
                    InstrumentCommand::Set(ParameterId(1), 700.0),
                    InstrumentCommand::Set(ParameterId(1), 800.0),
                    InstrumentCommand::Set(ParameterId(1), 900.0),
                    InstrumentCommand::Set(ParameterId(1), 1000.0),
                ],
            },
        )
    }

    #[test]
    fn emits_command_when_active() {
        let sut = get_sut_single_command(TimelineRange {
            start: 0.0,
            end: 1.0,
        });

        // one beat
        let range = TimelineRange {
            start: 0.0,
            end: 1.0,
        };

        let commands = sut.commands_between(range);

        assert_eq!(1, commands.len())
    }

    #[test]
    fn does_not_emit_commands_when_range_before() {
        let sut = get_sut_single_command(TimelineRange {
            start: 1.0,
            end: 2.0,
        });

        // one beat
        let range = TimelineRange {
            start: 0.0,
            end: 1.0,
        };

        let commands = sut.commands_between(range);

        assert_eq!(0, commands.len())
    }

    #[test]
    fn does_not_emit_commands_when_range_after() {
        let sut = get_sut_single_command(TimelineRange {
            start: 1.0,
            end: 2.0,
        });

        // one beat
        let range = TimelineRange {
            start: 2.0,
            end: 3.0,
        };

        let commands = sut.commands_between(range);

        assert_eq!(0, commands.len())
    }

    #[test]
    fn respects_interval() {
        let sut = get_sut_10_commands(
            TimelineRange {
                start: 0.0,
                end: 10.0,
            },
            0.5,
        ); // half beat period

        // one beat
        let range1 = TimelineRange {
            start: 0.0,
            end: 1.0,
        };
        // from the first to second beat, we expect the two first commands

        let commands1 = sut.commands_between(range1);

        assert_eq!(2, commands1.len());
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 100.0), commands1[0]);
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 200.0), commands1[1]);

        let range2 = TimelineRange {
            start: 1.0,
            end: 2.0,
        }; // from second to third beat, we expect the next two commands

        let commands2 = sut.commands_between(range2);

        assert_eq!(2, commands2.len());
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 300.0), commands2[0]);
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 400.0), commands2[1]);
    }

    #[test]
    fn repeats_pattern() {
        let sut = get_sut_single_command(TimelineRange {
            start: 0.0,
            end: 10.0,
        });

        // one beat
        let range = TimelineRange {
            start: 0.0,
            end: 4.0,
        };
        // from the first to fourth beat, we expect 4 times the same command

        let commands = sut.commands_between(range);

        assert_eq!(4, commands.len());
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 100.0), commands[0]);
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 100.0), commands[1]);
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 100.0), commands[2]);
        assert_eq!(InstrumentCommand::Set(ParameterId(1), 100.0), commands[3]);
    }

    #[test]
    fn handles_empty_pattern() {
        let sut = Clip::new(
            TimelineRange {
                start: 0.0,
                end: 10.0,
            },
            get_dummy_instrument_id(),
            Pattern {
                period: 1.0,
                commands: vec![],
            },
        );

        // one beat
        let range = TimelineRange {
            start: 0.0,
            end: 4.0,
        };
        // from the first to fourth beat, we expect no commands (empty pattern)

        let commands = sut.commands_between(range);

        assert_eq!(0, commands.len());
    }
}
