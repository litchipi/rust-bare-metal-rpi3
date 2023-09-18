pub trait SoundSource {
    /// Tick at every iteration of sample rate
    /// Count = u32::MIN: Start of the period
    /// Count = u32::MAX: End of the period
    /// Return: the signal value.
    ///   Ret u32::MIN = Min value of signal
    ///   Ret u32::MAX = Max value of signal
    fn tick(&self, count: u32) -> u32;
}

pub enum WaveShape {
    Sinus,
    Square,
    Saw,
    Noise,
}

impl WaveShape {
    fn tick(&self, count: u32, phase: u32) -> u32 {
        // TODO    Generate the raw wave ranging from u32::MIN to u32::MAX
        120
    }
}

pub struct Oscillator {
    pub shape: WaveShape,
    pub reversed_shape: bool,
    pub phase: u32,    // u32::MIN = 0        u32::MAX = 2pi

    pub adsr: AdsrHandler,

    pub volume: u32,
}

impl SoundSource for Oscillator {
    fn tick(&self, mut count: u32) -> u32 {
        // Get the value from waveshape
        if self.reversed_shape {
            count = u32::MAX - count;
        }
        let mut sig = self.shape.tick(count, self.phase);

        self.adsr.feed(&mut sig);

        // TODO    Apply volume reduction

        sig
    }
}

pub struct OscillatorMix(pub Vec<Oscillator>);

impl OscillatorMix {
    pub fn even_volume(&mut self) {
        let volume = u32::MAX / self.0.len();
        for osc in self.0.iter_mut() {
            osc.volume = volume;
        }
    }
}

impl SoundSource for OscillatorMix {
    fn tick(&self, count: u32) -> u32 {
        self.0.iter().map(|o| o.tick(count)).sum()
    }
}

pub struct AdsrHandler {
    pub attack: u32,
    pub decay: u32,
    pub sustain: u32,
    pub release: u32,

    // Some internal counters, buffers, etc ...
}

impl AdsrHandler {
    pub fn feed(&self, sig: &mut u32) {
        // TODO    Apply Attack
        // TODO    Apply Decay
        // TODO    Apply Sustain
        // TODO    Apply Release
    }
}
