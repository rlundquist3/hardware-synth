# hardware-synth

### Work in Progress

Firmware for a synthesizer, written to run on the [Daisy Seed 3](https://daisy.audio/products/seed3).

### Hardware

- [Daisy Seed 3](https://daisy.audio/products/seed3)
- USB-A Port on USB D-/D+ pins (29/30) with 5V (board is 3.3V, so needs separate power)
- audio out (e.g. TRS) on analog audio out pins (18 L/19 R)
- OLED display on SCL/SDA pins (12/13) using StemmaQT

### Running

As written, requires `cargo`, `just`, and `dfu-util`

The [Daisy Bootloader](https://github.com/electro-smith/DaisyBootloader/blob/main/dist/dsy_bootloader_v6_4-intdfu-2000ms.bin) is stored in flash, which pulls the application code in from memory.

To flash the bootloader:

- hold "BOOT" on the board
- press and release "RESET" while holding "BOOT"
- `just flash-bootloader`

Once the bootloader is flashed, to flash the firmware:

- hold "RESET" for a couple seconds and release
- the "USER" LED should pulse
- hold the "BOOT" button
- `just flash-firmware`
- once that starts going, "BOOT" can be released (i.e. not necessary to wait for it to finish)

**Could be flashed using `probe-rs` with a debug probe connected to the Daisy's Cortex-10 pinout.**

---

### Logging/debugging

**The hardware as described above does not include a debug probe on the Daisy's Cortex-10 pinout, therefore the firmware is not set up for any fancy logging/debugging.**

Simple serial logging over UART on board pin 14 (`USART1Tx`), e.g. using a Raspberry Pi Debug Probe [in serial](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html#serial-connections).

- connect the serial logger's ground to ground
- connect the device's UART Rx to Daisy's pin 13 (`USART1Tx`)
- `ls /dev/tty.*`
  - figure out which one your device is
- `screen /dev/tty.{your-device} 115200` in a separate terminal
