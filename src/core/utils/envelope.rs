#[derive(Debug, PartialEq)]
enum EnvelopeStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

pub struct Envelope {
    // attack, in nanoseconds
    attack: u64,
    // decay, in nanoseconds
    decay: u64,
    // sustain, a volume 0.0 - 1.0
    sustain: f32,
    // release, in nanoseconds
    release: u64,

    // stage (ADSR) where we are currently at
    stage: EnvelopeStage,
    // elapsed, time elapsed in the envelope (starting at 0)
    elapsed: u64,
    // volume at which the release initiated
    volume_at_release: f32,
}

impl Envelope {
    pub fn new(attack: u64, decay: u64, sustain: f32, release: u64) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            stage: EnvelopeStage::Idle,
            elapsed: 0,
            volume_at_release: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.elapsed = 0;
    }

    pub fn release(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.volume_at_release = self.volume();

            self.stage = EnvelopeStage::Release;
            self.elapsed = 0;
        }
    }

    pub fn step(&mut self, time_window: u64) {
        if self.stage == EnvelopeStage::Idle {
            return;
        }

        self.elapsed += time_window;

        self.stage = self.get_stage();
    }

    // gets the current value of the envelope (0.0 - 1.0)
    // based on the state (stage + elapsed time)
    pub fn volume(&self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => 0.0,
            EnvelopeStage::Sustain => self.sustain,
            EnvelopeStage::Attack => self.elapsed as f32 / self.attack as f32,
            EnvelopeStage::Decay => {
                // slope from value at release towards sustain value
                1.0 - (1.0 - self.sustain) * (self.elapsed - self.attack) as f32 / self.decay as f32
            }
            EnvelopeStage::Release => {
                // slope from volume at release to zero
                self.volume_at_release
                    - self.volume_at_release * self.elapsed as f32 / self.release as f32
            }
        }
    }

    /// Update the attack to a new value (in nanoseconds)
    pub fn set_attack(&mut self, attack: u64) {
        self.attack = attack;
    }

    /// Update the decay to a new value (in nanoseconds)
    pub fn set_decay(&mut self, decay: u64) {
        self.decay = decay;
    }

    /// Update the sustain to a new value (in volume: 0.0 - 1.0)
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain;
    }

    /// Update the release to a new value (in nanoseconds)
    pub fn set_release(&mut self, release: u64) {
        self.release = release;
    }

    /// Find the stage based on the Elapsed time and the current ADSR settings
    fn get_stage(&self) -> EnvelopeStage {
        if self.stage == EnvelopeStage::Idle {
            EnvelopeStage::Idle
        } else if self.stage == EnvelopeStage::Release {
            if self.elapsed <= self.release {
                EnvelopeStage::Release
            } else {
                EnvelopeStage::Idle
            }
        } else if self.elapsed <= self.attack {
            EnvelopeStage::Attack
        } else if self.elapsed <= (self.attack + self.decay) {
            EnvelopeStage::Decay
        } else {
            EnvelopeStage::Sustain
        }
    }
}

#[cfg(test)]
mod envelope_tests {
    use super::*;

    #[test]
    fn new_initialises_envelope_to_idle() {
        let mut sut = Envelope::new(0, 0, 1.0, 0);

        assert_eq!(sut.stage, EnvelopeStage::Idle);
        assert_eq!(sut.elapsed, 0);
    }

    #[test]
    fn start_sets_stage_to_attack_elapsed_to_zero() {
        let mut sut = Envelope::new(0, 0, 1.0, 0);
        sut.start();

        assert_eq!(sut.stage, EnvelopeStage::Attack);
        assert_eq!(sut.elapsed, 0);
    }

    #[test]
    fn release_sets_stage_to_release_elapsed_to_zero() {
        let mut sut = Envelope::new(0, 0, 1.0, 0);
        sut.start();
        sut.release();

        assert_eq!(sut.stage, EnvelopeStage::Release);
        assert_eq!(sut.elapsed, 0);
    }

    #[test]
    fn step_does_not_increment_elapsed_when_idle() {
        let mut sut = Envelope::new(0, 0, 1.0, 0);

        sut.step(500_000_000); // 0.5 seconds

        assert_eq!(sut.elapsed, 0);
    }

    #[test]
    fn step_does_not_increment_elapsed_when_not_idle() {
        let mut sut = Envelope::new(2_000_000_000, 0, 1.0, 0);
        sut.start();

        sut.step(500_000_000); // 0.5 seconds
        assert_eq!(sut.elapsed, 500_000_000);
        sut.step(500_000_000); // 0.5 seconds
        assert_eq!(sut.elapsed, 1_000_000_000);
    }

    #[test]
    fn get_stage_without_start_returns_idle() {
        let sut = Envelope::new(2_000_000_000, 0, 1.0, 0);

        assert_eq!(sut.get_stage(), EnvelopeStage::Idle);
    }

    #[test]
    fn get_stage_after_start_returns_attack() {
        let mut sut = Envelope::new(2_000_000_000, 0, 1.0, 0);
        sut.start();

        assert_eq!(sut.get_stage(), EnvelopeStage::Attack);
    }

    #[test]
    fn get_stage_after_attack_returns_decay() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 1.0, 0);
        sut.start();

        sut.step(3_000_000_000);

        assert_eq!(sut.get_stage(), EnvelopeStage::Decay);
    }

    #[test]
    fn get_stage_after_decay_returns_sustain() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 1.0, 0);
        sut.start();

        sut.step(5_000_000_000);

        assert_eq!(sut.get_stage(), EnvelopeStage::Sustain);
    }

    #[test]
    fn get_stage_after_sustain_returns_idle() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 1.0, 2_000_000_000);
        sut.start();

        sut.release();
        sut.step(3_000_000_000);

        assert_eq!(sut.get_stage(), EnvelopeStage::Idle);
    }

    #[test]
    fn volume_after_50_pct_attack_elapsed_is_50_pct() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 1.0, 2_000_000_000);
        sut.start();

        sut.step(1_000_000_000); //  we must be half way through Attack

        assert_eq!(sut.stage, EnvelopeStage::Attack);
        assert_eq!(sut.volume(), 0.5);
    }

    #[test]
    fn volume_after_50_pct_decay_elapsed_is_75_pct() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 0.5, 2_000_000_000);
        sut.start();

        sut.step(3_000_000_000);

        assert_eq!(sut.stage, EnvelopeStage::Decay);

        // Decay goes from 1.0 to 0.5 (sustain) so halfway point should be 0.75
        assert_eq!(sut.volume(), 0.75);
    }

    #[test]
    fn volume_after_50_pct_release_elapsed_is_25_pct() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 0.5, 2_000_000_000);
        sut.start();

        sut.step(1_000_000_000); //  we move towards half the attack (so 0.5)

        sut.release();

        sut.step(1_000_000_000); //  we move towards half the release

        assert_eq!(sut.stage, EnvelopeStage::Release);

        // Decay goes from 0.5 to 0.0, so halfway point should be 0.25
        assert_eq!(sut.volume(), 0.25);
    }

    #[test]
    fn volume_after_100_pct_release_elapsed_is_0_pct() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 0.5, 2_000_000_000);
        sut.start();

        sut.step(1_000_000_000); //  we move towards half the attack (so 0.5)

        sut.release();

        sut.step(2_000_000_000); //  we move towards half the release

        assert_eq!(sut.stage, EnvelopeStage::Release);

        // Decay goes from 0.5 to 0.0, so halfway point should be 0.25
        assert_eq!(sut.volume(), 0.0);
    }

    #[test]
    fn volume_release_ends_should_be_zero() {
        let mut sut = Envelope::new(2_000_000_000, 2_000_000_000, 0.5, 2_000_000_000);
        sut.start();

        sut.release();

        sut.step(5_000_000_000); //  we move towards 5 seconds in the release

        assert_eq!(sut.stage, EnvelopeStage::Idle); // at this point we are idle

        // When idle, we expect 0.0
        assert_eq!(sut.volume(), 0.0);
    }
}
