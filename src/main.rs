use std::{
    env,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    config::config::Config,
    core::{
        commands::RackCommand::{self},
        update_loop::{UpdateLoop, UpdateLoopError},
        update_loop_app::DefaultApp,
        update_loop_config::UpdateLoopConfig,
    },
    rack::rack::Rack,
    sequencer::sequencer::Sequencer,
};

mod config;
mod core;
mod instruments;
mod rack;
mod sequencer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Missing mandatory argument <config.toml>")
    }

    let (tx, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();

    let config = Config::from_file(args[1].to_string()).unwrap();
    let mut rack = Rack::from_config(rx, &config).unwrap();
    let mut sequencer = Sequencer::from_config(tx, &config, &rack).unwrap();

    rack.play();
    sequencer.play();

    let mut app = DefaultApp::new(rack, sequencer);
    let update_loop = UpdateLoop::new(UpdateLoopConfig::default());

    match update_loop.run(&mut app) {
        Ok(()) => println!("Engine stopped normally"),
        Err(UpdateLoopError::NotStarted) => println!("Update Loop could not start"),
    }
}
