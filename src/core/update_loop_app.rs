use crate::{rack::rack::Rack, sequencer::sequencer::Sequencer};

pub enum AppState {
    Running,
    Finished,
}

pub trait App {
    fn update(&mut self, dt: u128);
    fn render(&mut self);

    fn state(&self) -> AppState;
}

pub struct DefaultApp {
    rack: Rack,
    sequencer: Sequencer,
    state: AppState,
}

impl DefaultApp {
    pub fn new(rack: Rack, sequencer: Sequencer) -> Self {
        Self {
            rack,
            sequencer,
            state: AppState::Running,
        }
    }
}

impl App for DefaultApp {
    fn update(&mut self, dt: u128) {
        // TODO: Ask the Sequencer for all events within this period
        // TODO: Calculate at which sample the event occurs
        // TODO: Pass this list to the rack update who will give it to each instrument

        let sample_count = (dt as f32 / 1_000_000_000.0) * self.rack.sample_rate as f32;
        if let Err(err) = self.rack.update(dt, sample_count as u32) {
            eprintln!("Rack update failed: {:?}", err);
            self.state = AppState::Finished;
        }

        if let Err(err) = self.sequencer.update(dt, sample_count as u32) {
            eprintln!("Sequencer update failed: {:?}", err);
            self.state = AppState::Finished;
        }
    }

    fn render(&mut self) {
        // Currently render does nothing yet
    }

    fn state(&self) -> AppState {
        AppState::Running
    }
}
