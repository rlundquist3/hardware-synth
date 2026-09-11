use alloc::format;
use embassy_stm32::peripherals::PA1;
use embassy_time::Timer;

use crate::{
    logger::serial_log,
    tinyusb::{
        tuh_midi_mount_cb_t, tuh_midi_stream_read, tuh_task_ext, tusb_rhport_init,
        tusb_rhport_init_t, tusb_role_t_TUSB_ROLE_HOST, tusb_speed_t_TUSB_SPEED_FULL,
    },
};

pub mod notes;

#[embassy_executor::task]
pub async fn usb_host_task() {
    loop {
        unsafe {
            tuh_task_ext(u32::MAX, false);
            serial_log("TUH TASK EXT");
        }
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
pub async fn initialize_midi_host() {
    serial_log("Initializing MIDI Host...");

    let host_init = tusb_rhport_init_t {
        role: tusb_role_t_TUSB_ROLE_HOST,
        speed: tusb_speed_t_TUSB_SPEED_FULL,
    };

    unsafe {
        tusb_rhport_init(0, &host_init);
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_mount_cb(idx: u8, mount_cb_data: *const tuh_midi_mount_cb_t) {
    let data = unsafe { &*mount_cb_data };
    serial_log(&format!(
        "MIDI mounted: index = {}, address = {}, rx cable count={}, tx cable count = {}",
        idx, data.daddr, data.rx_cable_count, data.tx_cable_count,
    ));
}

#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_umount_cb(idx: u8) {
    serial_log(&format!("MIDI interface index = {} unmounted", idx));
}

#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_rx_cb(idx: u8, xferred_bytes: u32) {
    if xferred_bytes == 0 {
        return;
    }

    let mut cable_num: u8 = 0;
    let mut buf: [u8; 48] = [0; 48];

    loop {
        let byte_read: u32 = unsafe {
            tuh_midi_stream_read(idx, &mut cable_num, buf.as_mut_ptr(), buf.len() as u16)
        };

        if byte_read == 0 {
            break;
        }

        serial_log(&format!("Byte: {:02x}", byte_read));
    }
    serial_log("MIDI Cable {} rx: ");
    for byte in buf {
        serial_log(&format!("{:02x}", byte));
    }
}

// TODO: remove these
#[unsafe(no_mangle)]
pub extern "C" fn tuh_mount_cb(daddr: u8) {
    serial_log(&format!("USB device {} mounted", daddr));
}
#[unsafe(no_mangle)]
pub extern "C" fn tuh_unmount_cb(daddr: u8) {
    serial_log(&format!("USB device {} unmounted", daddr));
}

#[unsafe(no_mangle)]
pub static SystemCoreClock: u32 = 480_000_000;

#[unsafe(no_mangle)]
pub extern "C" fn tusb_time_millis_api() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}
