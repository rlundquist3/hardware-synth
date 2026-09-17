flash-firmware:
    cargo objcopy --release -- -O binary firmware.bin
    dfu-util -a 0 -s 0x90040000:leave -D firmware.bin -d ,0483:df11

flash-bootloader:
    dfu-util -a 0 -s 0x08000000:leave -D dsy_bootloader_v6_4-intdfu-2000ms.bin -d ,0483:df11

test-core:
    cargo test -p synth-core --target aarch64-apple-darwin

logs:
    screen /dev/tty.usbmodem1302 115200

docs:
    cargo doc --open --document-private-items