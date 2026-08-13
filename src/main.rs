use std::{
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    config::config::Config, core::{
        commands::RackCommand::{self}, rack::rack::Rack, update_loop::{UpdateLoop, UpdateLoopError}, update_loop_app::DefaultApp, update_loop_config::UpdateLoopConfig,
    }, sequencer::sequencer::Sequencer,
};

mod config;
mod core;
mod instruments;
mod sequencer;

fn main() {
    let (tx, rx): (Sender<RackCommand>, Receiver<RackCommand>) = mpsc::channel();
    
    let config = Config::from_file("test_files/happy_birthday.toml".to_string()).unwrap();
    let mut rack = Rack::from_config(rx, &config).unwrap();
    let mut sequencer = Sequencer::from_config(tx, &config, &rack).unwrap();

    rack.play();
    sequencer.play();

    let mut app = DefaultApp::new(rack, 48000, sequencer);
    let update_loop = UpdateLoop::new(UpdateLoopConfig::default());

    match update_loop.run(&mut app) {
        Ok(()) => println!("Engine stopped normally"),
        Err(UpdateLoopError::NotStarted) => println!("Update Loop could not start"),
    }
}
