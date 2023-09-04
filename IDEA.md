# IDEA

Jam helper

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
- Use audio jack onboard:
  - Pin 40: Right audio
  - Pin 41: Left audio


## Resources

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
