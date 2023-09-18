static instrument: Instrument = SecureLock::new(Instrument::init());

static refresh: bool = SecureLock::new(true);

fn sample_timer_irq() {
    refresh.write() = true;
}

fn setup_audio() {
    configure_pin(40, AltFunc0);
    configure_pin(41, AltFunc0);

    // Enable channels
    PWEN1.write(1);
    PWEN2.write(2);
    
    // Set PWM algorithm
    MSEN1.write(0);
    MSEN2.write(0);

    // Set PWM mode
    MODE1.write(0);
    MODE2.write(0);

    // Use Data register
    USEF1.write(0);
    USEF2.write(0);

    // Set rest bit to 0
    SBIT1.write(0);
    SBIT2.write(0);
}

fn update_audio() {
    let (left, right) = instrument.read().tick();
    DAT1.write(right);
    DAT2.write(left);
}

fn main() {
    loop {

        let r = { refresh.read(); };
        if r {
            refresh.write() = false;
            // Get state of buttons and inputs
            refresh_inputs();

            // Depending on the state of the interface and the buttons pressed,
            // Update the settings on the instrument
            refresh_instrument();
            
            update_audio();

            // Create a task for an other CPU to fill the framebuffer
            // and interface with the GPU to render the screen
            render_interface();
        }

        // If no refresh since last check, wait for interrupts
        if !refresh.read() {
            wfi();
        }
    }
}
