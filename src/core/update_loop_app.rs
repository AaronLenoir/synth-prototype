use crate::{
    rack::rack::Rack,
    sequencer::{event::RackEvent, sequencer::Sequencer},
};

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
    /// Each iteration of the update loop this function is called it will do two things
    /// It will trigger the generation of audio for a specific time window
    ///
    ///   - Check which events the sequencer wants to execute in this time window
    ///   - Call the Rack update function - who will internally update the instruments in the
    ///     appropriate order
    fn update(&mut self, dt: u128) {
        let sample_count = (dt as f32 / 1_000_000_000.0) * self.rack.sample_rate as f32;

        let events: Vec<RackEvent> = match self.sequencer.step(dt, sample_count as u32) {
            Ok(events) => events,
            Err(err) => {
                eprintln!("Rack update failed: {:?}", err);
                self.state = AppState::Finished;
                return;
            }
        };

        if let Err(err) = self.rack.update(dt, sample_count as u32, events) {
            eprintln!("Rack update failed: {:?}", err);
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
