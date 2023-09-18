// Interface
// - Live / Menu Switch
// - One big infinite rotary knob
// - One big red button, with RGB light inside
// - 5 lit buttons with an RGB LED under

// Most of the work is done through MIDI channels

static interface: Interface = SecureLock::new(Interface::new());
static LOAD_PRESET_INTERFACE: InstrumentPresetLoading = SecureLock::new(InstrumentPresetLoading::new());

pub enum InterfaceInput {
    LiveMenuSwitch(bool),
    RotaryKnow(u8, bool),    // By how much, on which side
    BigButton,

    KeyButton(usize),
} 

pub struct Interface {
    pub active: u8,

    live: LiveInterface,
    options: OptionsInterface,
}

impl Interface {
    pub fn init(&mut self) {
        // init self
        LOAD_PRESET_INTERFACE.write().load_instruments_preset_bank()
    }

    pub fn render(&self) {
        match self.active {
            0 => self.live.render(),
            1 => self.options.render(),
            _ => unreachable!(),
        }
    }

    pub fn react_input_event(&mut self, input: InterfaceInput) {
        match self.active {
            0 => self.live.react_input_event(input),
            1 => self.options.react_input_event(input),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub enum LiveInterfaceState {
    Live,
    SelectedChannel(usize),
    LoadPreset(usize),
}

impl LiveInterfaceState {
    pub fn react_input_event(self, iface: &mut LiveInterface, event: InterfaceInput) -> LiveInterfaceState {
        match self {
            LiveInterfaceState::Live => match event {
                InterfaceInput::KeyButton(1) => return LiveInterfaceState::SelectedChannel(iface.hover),
                InterfaceInput::RotaryKnob(_, next) => {
                    if next {
                        iface.hover_next();
                    } else {
                        iface.hover_prev();
                    }
                },
            },

            LiveInterfaceState::SelectedChannel(n) => match event {
                InterfaceInput::KeyButton(0) => return LiveInterfaceState::Live,
                InterfaceInput::KeyButton(1) => return LiveInterfaceState::LoadPreset(n),
                _ => {},
            },

            LiveInterfaceState::LoadPreset(n) => match event {
                InterfaceInput::KeyButton(0) => LiveInterfaceState::SelectedChannel(n),
                InterfaceInput::KeyButton(1) => {
                    let preset = LOAD_PRESET_INTERFACE.read().load_preset();
                    let idx = iface.selected.unwrap();
                    let channel = iface.channels.get_mut(idx).unwrap();
                    channel.instrument = preset;
                    return LiveInterfaceState::Live
                },

                InterfaceInput::RotaryKnob(_, next) => {
                    if next {
                        LOAD_PRESET_INTERFACE.write().next_preset();
                    } else {
                        LOAD_PRESET_INTERFACE.write().prev_preset();
                    }
                }
                _ => {},
            } 
        }

        self
    }
}

pub struct LiveInterface {
    channels: Vec<LiveChannel>,
    hover: usize,

    state: LiveInterfaceState,
}

impl LiveInterface {
    fn hover_next(&mut self) {
        self.hover += 1;
        if self.hover >= self.channels.len() {
            self.hover = 0;
        }
    }

    fn hover_prev(&mut self) {
        if self.hover == 0 {
            self.hover = self.channels.len();
        }
        self.hover -= 1;
    }

    pub fn render(&self) {
        if self.load_preset_window {
            LOAD_PRESET_INTERFACE.render();
        } else {
            // Render self
        }
    }

    pub fn react_input_event(&mut self, event: InterfaceInput) {
        let next_state = self.state.clone().react_input_event(&mut self, event);
        self.state = next_state;
    }
}

pub struct InstrumentPresetLoading {
    selected_preset: usize,
    instrument_presets_bank: Vec<String>,
}

impl InstrumentPresetLoading {
    pub fn load_instruments_preset_bank(&mut self) {
        // Load from SD card
    }

    pub fn load_preset(&self) -> Instrument {
        todo!();
    }

    pub fn next_preset(&mut self) {
        self.selected_preset += 1;
        if self.selected_preset >= self.instrument_presets_bank.len() {
            self.selected_preset = 0;
        }
    }

    pub fn prev_preset(&mut self) {
        if self.selected_preset == 0 {
            self.selected_preset = self.instrument_presets_bank.len();
        }
        self.selected_preset -= 1;
    }

    pub fn render(&self) {
    }
}
