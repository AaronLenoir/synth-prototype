use std::{
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use rand::{RngExt, seq};

use crate::{
    config::{config::Config, rack_builder::RackBuilder, rack_config::RackConfig},
    core::{
        commands::{
            InstrumentCommand,
            RackCommand::{self},
        },
        connection::{Connection, EndPoint},
        rack::{InstrumentId, Rack},
        update_loop::{UpdateLoop, UpdateLoopError},
        update_loop_app::DefaultApp,
        update_loop_config::UpdateLoopConfig,
    },
    instruments::{
        audio_out::AudioOutPorts,
        mixer::{Mixer, MixerInPorts, MixerOutPorts},
        random_generator::{RandomGenerator, RandomGeneratorPorts},
        signal_source::{SignalSource, SignalSourceParameters, SignalSourcePorts},
    },
    sequencer::{
        clip::Clip, meter::Meter, pattern::Pattern, sequencer::Sequencer,
        timeline_range::TimelineRange,
    },
};

mod config;
mod core;
mod instruments;
mod sequencer;

fn main() {
    let (tx, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();

    let mut rack = Rack::from_config_file(rx, "test_files/two_sines.toml".to_string()).unwrap();

    let mut sequencer = Sequencer::new(
        120,
        Meter {
            numerator: 4,
            denominator: 4,
        },
        tx,
    );

    rack.play();
    sequencer.play();

    let id = rack.instrument_id("generator1").unwrap();
    let clip1 = Clip::new(
        TimelineRange {
            start: 0.0,
            end: 8.0,
        },
        id,
        Pattern {
            period: 1.0,
            commands: vec![
                InstrumentCommand::Set(SignalSourceParameters::FREQUENCY, 300.0),
                InstrumentCommand::Set(SignalSourceParameters::FREQUENCY, 330.0),
            ],
        },
    );
    let clip2 = Clip::new(
        TimelineRange {
            start: 8.0,
            end: 160.0,
        },
        id,
        Pattern {
            period: 0.5,
            commands: vec![
                InstrumentCommand::Set(SignalSourceParameters::FREQUENCY, 300.0),
                InstrumentCommand::Set(SignalSourceParameters::FREQUENCY, 330.0),
            ],
        },
    );
    sequencer.add_clip(clip1);
    sequencer.add_clip(clip2);

    let mut app = DefaultApp::new(rack, 48000, sequencer);
    let update_loop = UpdateLoop::new(UpdateLoopConfig::default());

    match update_loop.run(&mut app) {
        Ok(()) => println!("Engine stopped normally"),
        Err(UpdateLoopError::NotStarted) => println!("Update Loop could not start"),
    }
}
