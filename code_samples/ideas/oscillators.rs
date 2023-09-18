pub trait SoundSource {
    /// Tick at every iteration of sample rate
    /// Count = u32::MIN: Start of the period
    /// Count = u32::MAX: End of the period
    /// Return: the signal value.
    ///   Ret u32::MIN = Min value of signal
    ///   Ret u32::MAX = Max value of signal
    fn tick(&self, count: u32) -> u32;
}

#[derive(Serialize, Deserialize)]
pub enum WaveShape {
    Sinus,
    Square,
    Saw,
    Noise,
}

impl WaveShape {
    fn tick(&self, count: u32, phase: u32, velocity: u32) -> u32 {
        // TODO    Generate the raw wave ranging from u32::MIN to u32::MAX

        if let Some(vel) = self.velocity {
            // TODO    Apply velocity reduction
            // Or change the wave depending on velocity
        }

        120
    }
}

#[derive(Serialize, Deserialize)]
pub struct Oscillator {
    pub shape: WaveShape,
    pub reversed_shape: bool,
    pub phase: u32,    // u32::MIN = 0        u32::MAX = 2pi

    pub velocity: Option<u32>,
    pub volume: u32,
}

impl SoundSource for Oscillator {
    fn tick(&self, mut count: u32) -> u32 {
        // Get the value from waveshape
        if self.reversed_shape {
            count = u32::MAX - count;
        }
        let mut sig = self.shape.tick(count, self.phase, self.velocity);

        // TODO    Apply volume reduction
        sig
    }
}

#[derive(Serialize, Deserialize)]
pub struct AdsrHandler {
    pub sig_max: u32,

    // TODO       Get MAX time limit for which u32::MAX corresponds to
    pub attack: u32,
    pub decay: u32,
    pub sustain: u32,
    pub release: u32,
}

impl AdsrHandler {
    pub fn feed(&mut self, sig: u32, pressed: bool) -> u32 {
        // TODO    Apply Attack
        // TODO    Apply Decay
        // TODO    Apply Sustain
        // TODO    Apply Release
        sig
    }
}
