# IDEA

Jam helper on my **Raspberry Pi 3 Model B rev 1.2**

## Features

- Translate volca tempo signal to keystep tempo signal
- Bank of long samples
  - Auto-adapt the speed of the samples to the tempo
  - Loop sample
  - Add FX on the samples
  - Apply effects in coordination with the tempo
- MIDI muxer
  - Get MIDI input from Keystep
  - Redirect to a specific MIDI based on MIDI channel set
  - Apply special rules on MIDI events based on who is connected

## Implementation

Global state under a lock system
Everything happens with IRQ

Sampling IRQ is the fastest (~ 48kHz ?)

Main loop will update things on the global state based on the messages it received on its channel
Then will just WFE

### Timers

Have a `Timer` struct that interfaces with the system Timer IRQ
This struct only modified during Timer IRQ

```
pub struct Timer {
  registered_timers: RwLock<Vec<u64>>,
}
impl Timer {
  // Called inside the Timer IRQ handler
  fn tick(&self) {
    for time_left in self.registered_timers.write().iter_mut() {
      *time_left = time_left.saturating_sub(TIMER_RESOLUTION_US);
    }
  }

  // Register new timer to follow, return the number
  fn register_new(&self, time_us: u64) -> usize {
    let idx = self.registered_timers.read().len();
    assert!((idx + 1) < MAX_TIMERS_COUNT, "Max timers count reached");
    self.registered_timers.write().push(time_us);
    idx
  }

  fn set(&self, timer_nb: usize, time_us: u64) {
    match self.registered_timers.write().get_mut(&timer_nb) {
      Some(timer) => *timer = time_us,
      None => panic!("Attempt to set timer {timer_nb}, but it doesn't exist"),
    }
  }

  // Get remaining time to wait (in microseconds)
  fn get(&self, timer_nb: usize) -> u64 {
    if let Some(rest) = self.registered_timers.read().get(timer_nb) {
      rest
    } else {
      panic!("Unable to get timer {timer_nb}: Not there");
    }
  }

  fn wait(&self, time_us: u64) {
    let nb = self.register_new(time_us);
    while self.get(nb) > 0 {
      asm::wfi();
    }
  }
}
```

Do not use this struct for sampling or screen rendering, but everything else is fine

### Translate tempo signal between volca and keystep

IRQ on tempo rise, update the tempo state variable (based on duration since last IRQ)
Every iteration of the sampling IRQ, produce peak if needed on the tempo output

### Bank of long samples

- Have some metadata on the samples:
  - Tempo
  - Sample rate
  - Format
  - Total duration
- Saved on SD card
- Use audio jack onboard

## Board support package

Write basic library as a wrapper around the raw hardware registers

Embedded-hal compliant for pins definition

Arduino-like for pins usage:

```
fn set_pin(number)
fn clear_pin(number)
fn toggle_pin(number)
fn uart_start(tx_pin, rx_pin, speed)
fn init()

fn set_irq(pin_number, event, handler_function)  // Event: Rising edge, falling edge, etc ...
```

### Pinouts

```
Bank 0:  P0 to P27
Bank 1:  P28 to P45
Bank 2:  P46 to P57 
```

```
P14: TX UART 0
P15: RX UART 0

P40: Right audio
P41: Left Audio

P42: Ethernet CLK
P43: Wifi CLK

P44: I2C 0 SDA
P45: I2C 0 SCL

P46: SMPS SCL
P47: SMPS SDA

P48: SD CLK
P49: SD CMD
P50: SD D0
P51: SD D1
P52: SD D2
P53: SD D3
```

```
Set pin:
  Write 1 >> n (based on pin number 0..31) to GPSET0
  or 
  Write 1 >> (n-32) (based on pin number 32..57) to GPSET1

Clear pin:
  Write 1 >> n (based on pin number 0..31) to GPCLR0
  or 
  Write 1 >> (n-32) (based on pin number 32..57) to GPCLR1
```

## Resources

- [RPI3FXProc](https://github.com/rahealy/rpi3fxproc)
- [Low Level Devel - HDMI](https://www.youtube.com/watch?v=DxAxlc5Ldt4)
- [Low Level Devel - Video with DMA](https://www.youtube.com/watch?v=4JtZQ88x5_c)
- [Use GPU](https://github.com/BrianSidebotham/arm-tutorial-rpi/blob/master/part-5/readme.md)

## SMP

At boot:
```
  if nb_cpu == 0 {     // Primary CPU
    kernel_init();
  } else {
    loop {
      WFE()
      WFI()
      if variable_set(MEMORY_ADDR[nb_cpu]) {
        execute_function(MEMORY_ADDR[nb_cpu]);
      }
    }
  }
```

Translate this into ARM assembly, and test passing Rust functions to it using the primary CPU.

Then, create a basic scheduler:
- Interruptions:
  - Generates operations requests
- Primary CPU loop:
  - Execute any operation request it can on its own: 
    - Generate new timer IRQs
    - Starts / stop a new process
  - Send operations requests to any free CPU, or wait for one to be freed before
  - Then go back to WFE / WFI

## MIDI

Get MIDI messages from UART
When connected to PC:
- Can send data over UART to test

Uses baudrate of 31250
