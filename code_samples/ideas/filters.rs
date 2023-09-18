pub trait Filter {
    fn feed(&self, sig: u32) -> u32;
    fn set_cutoff(&mut self, cutoff: u32);
    fn get_cutoff(&mut self, cutoff: u32);
    fn set_peak(&mut self, peak: u32);
    fn get_peak(&mut self, peak: u32);
    fn set_att_level(&mut self, lvl: u32);
    fn get_att_level(&mut self, lvl: u32);
}

pub struct LowPassFilter {
    pub cutoff: u32,    // TODO:    Determine low and high bands
    pub peak: u32,
    pub level: u32,
}

impl Filter for LowPassFilter {
    fn feed(&self, sig: u32) -> u32 {
        
    }

    fn set_cutoff(&mut self, cutoff: u32) {
        self.cutoff = cutoff;
    }

    fn set_peak(&mut self, peak: u32) {
        self.peak = peak;
    }

    fn set_att_level(&mut self, lvl: u32) {
        self.level = lvl;
    }
}
