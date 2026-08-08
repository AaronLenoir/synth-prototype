use crate::core::update_loop_app::App;
use crate::core::update_loop_app::AppState;
use crate::core::update_loop_config::UpdateLoopConfig;

use core::time::Duration;
use quanta::Clock;
use quanta::Instant;
use std::thread::sleep;

const DEFAULT_STATE_CHECK_INTERVAL: u128 = 1_000_000_000; //  1 s

pub struct UpdateLoop {
    config: UpdateLoopConfig,
}

pub enum UpdateLoopError {
    NotStarted,
}

impl UpdateLoop {
    pub fn new(config: UpdateLoopConfig) -> Self {
        UpdateLoop { config }
    }

    pub fn run<A: App>(&self, app: &mut A) -> Result<(), UpdateLoopError> {
        let step = self.config.step_in_nanos; // 1/60 second in nanoseconds
        let clamp = self.config.accumulator_cap_in_nanos; // 1 s

        let state_check_interval = step.max(DEFAULT_STATE_CHECK_INTERVAL);
        let mut elapsed_since_state_check: u128 = 0;

        let clock = Clock::new();

        let mut last = clock.now();
        let mut accumulator: u128 = 0;

        let mut expected_frame_time = clock.now(); // time we expect next frame

        loop {
            let now = clock.now();

            let frame_time = (now - last).as_nanos();
            last = now;

            // every iteration, add the elapsed time into the accumulator
            accumulator += frame_time;
            if accumulator > clamp {
                accumulator = clamp;
            }

            while accumulator >= step {
                // call the simulation step here
                app.update(step);

                accumulator -= step;
            }

            // call the renderer here
            app.render();

            // check frame time
            self.cap_fps(&clock, &mut expected_frame_time);

            // check state (sometimes)
            elapsed_since_state_check += frame_time;
            if elapsed_since_state_check >= state_check_interval {
                if let AppState::Finished = app.state() {
                    break;
                }
                elapsed_since_state_check = 0;
            }
        }

        Ok(())
    }

    fn cap_fps(&self, clock: &Clock, expected_frame_time: &mut Instant) {
        let fps_cap = Duration::from_millis(self.config.fps_cap_in_ms); // 60 FPS Cap 
        let now = clock.now();
        let next = *expected_frame_time + fps_cap; // when we expect the next frame

        if next > now {
            // we have time for the next frame, let's get some sleep
            *expected_frame_time = next;
            sleep(next - now);
        } else {
            // next frame should've happened, can't sleep now ...
            *expected_frame_time = now;
        }
    }
}
