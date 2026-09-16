flash-firmware:
    cargo objcopy --release -- -O binary firmware.bin
    dfu-util -a 0 -s 0x90040000:leave -D firmware.bin -d ,0483:df11

flash-bootloader:
    dfu-util -a 0 -s 0x08000000:leave -D dsy_bootloader_v6_4-intdfu-2000ms.bin -d ,0483:df11