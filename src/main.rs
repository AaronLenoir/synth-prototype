// Configurable audio generator in RUST for educational purposes
// Copyright (c) 2026, Aaron Lenoir

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env;

use crate::{
    config::config::Config,
    core::{
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

    let config = Config::from_file(args[1].to_string()).unwrap();
    let mut rack = Rack::from_config(&config).unwrap();
    let mut sequencer = Sequencer::from_config(&config, &rack).unwrap();

    rack.play();
    sequencer.play();

    let mut app = DefaultApp::new(rack, sequencer);
    let update_loop = UpdateLoop::new(UpdateLoopConfig::default());

    match update_loop.run(&mut app) {
        Ok(()) => println!("Engine stopped normally"),
        Err(UpdateLoopError::NotStarted) => println!("Update Loop could not start"),
    }
}
