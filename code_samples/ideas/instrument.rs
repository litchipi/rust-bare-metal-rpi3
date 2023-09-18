pub struct Instrument {
    oscillators: Vec<Oscillator>,
    filter: BTreeMap<usize, Box<impl Filter>>,
    osc_adsr: BTreeMap<usize, AdsrHandler>,
    flt_adsr: BTreeMap<usize, AdsrHandler>,
    balance: BTreeMap<usize, i32>,    // i32::MIN => Left,     i32::MAX => Right

    pressed: bool,
    count_incr: u32,
    count: u32,
}

impl Instrument {

    /// Interactions

    pub fn press(&mut self, velocity: u32, tick_per_period: u32) {
        self.pressed = true;
        self.count = 0;
        self.count_incr = u32::MAX / tick_per_period;
        self.oscillators.iter_mut().for_each(|o| o.velocity = Some(velocity));
    }

    pub fn release(&mut self) {
        self.pressed = false;
        self.oscillators.iter_mut().for_each(|o| o.velocity = None);
    }

    ///    Build the instrument

    pub fn add_oscillator(&mut self) {
        self.oscillators.push(Oscillator::default());
    }

    pub fn add_filter(&mut self, osc_idx: usize, filter: Box<impl Filter>) {
        assert!(osc_idx < oscillators.len());
        self.filters.insert(osc_idx, filter);
    }

    pub fn add_osc_adsr(&mut self, osc_idx: usize, adsr: AdsrHandler) {
        assert!(osc_idx < oscillators.len());
        self.osc_adsr.insert(osc_idx, adsr);
    }

    pub fn add_flt_adsr(&mut self, osc_idx: usize, adsr: AdsrHandler) {
        assert!(osc_idx < oscillators.len());
        assert!(self.filters.contains_key(&osc_idx));
        self.flt_adsr.insert(osc_idx, adsr);
    }

    pub fn set_osc_balance(&mut self, osc_idx: usize, balance: i32) {
        assert!(osc_idx < oscillators.len());
        self.balance.insert(osc_idx, balance);
    }

    /// Internal
    
    fn tick_nb(&self, n: usize, count: u32, isleft: bool) -> u32 {
        let osc = self.oscillators.get_mut(n).unwrap();

        let mut sig = osc.tick(count);

        // TODO    Adapt volume of oscillator based on its balance

        if let Some(osc_adsr) = self.osc_adsr.get_mut(n) {
            sig = osc_adsr.feed(sig, self.pressed);
        }

        if let Some(filter) = self.filter.get_mut(n) {
            let mut cutoff = filter.get_cutoff();
            if let Some(flt_adsr) = self.flt_adsr.get_mut(n) {
                cutoff = flt_adsr.feed(cutoff, self.pressed);
                filter.set_cutoff(cutoff)
            }
            sig = filter.feed(sig)
        }

        sig
    }

    fn apply_fx(&self, sigl: u32, sigr: u32) -> (u32, u32) {
        (sigl, sigr)
    }

    fn tick(&self) -> (u32, u32) {
        let mut resl = 0;
        let mut resr = 0;
        for n in 0..self.oscillators.len() {
            resl += self.tick_nb(n, self.count, true);
            resr += self.tick_nb(n, self.count, false);
        }
        self.count = self.count.wrapping_add(self.count_incr);
        self.apply_fx(resl, resr)
    }
}
