use alloc::format;
use core::{
    mem,
    sync::atomic::{AtomicBool, Ordering},
};
use embassy_stm32::{
    Peri,
    gpio::{AfType, Flex, OutputType, Speed},
    pac,
    peripherals::{PB14, PB15, USB_OTG_HS},
    rcc,
};
use embassy_time::Timer;

use crate::{
    logger::{serial_error, serial_log},
    tinyusb::{
        BOARD_TUH_RHPORT, tuh_midi_mount_cb_t, tuh_midi_stream_read, tuh_rhport_reset_bus,
        tuh_task_ext, tusb_rhport_init, tusb_rhport_init_t, tusb_role_t_TUSB_ROLE_HOST,
        tusb_speed_t_TUSB_SPEED_FULL,
    },
};

pub mod notes;

static MIDI_MOUNTED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn usb_host_task() {
    let mut stalled_ticks: u32 = 0;

    loop {
        unsafe {
            tuh_task_ext(u32::MAX, false);
        }

        /*
         * If a controller was plugged in at boot and attempted to connect before
         * the host started, it won't be mounted. Wait ~3s and reset bus to mount
         * controllers in these cases. (Only hit if something is plugged in)
         */
        let hprt = pac::USB_OTG_HS.hprt().read();
        if hprt.pcsts() && hprt.pena() && !MIDI_MOUNTED.load(Ordering::Relaxed) {
            stalled_ticks += 1;
            if stalled_ticks >= 15_000
            /* 15k * 200 micro secs = 3s */
            {
                stalled_ticks = 0;
                serial_log("USB: enumeration stalled, resetting bus");
                unsafe {
                    tuh_rhport_reset_bus(BOARD_TUH_RHPORT, true);
                }
                Timer::after_millis(20).await;
                unsafe {
                    tuh_rhport_reset_bus(BOARD_TUH_RHPORT, false);
                }
            }
        } else {
            stalled_ticks = 0;
        }

        // Timeout to yield to lower-priority tasks
        Timer::after_micros(200).await;
    }
}

pub fn initialize_midi_host(dn: Peri<'static, PB14>, dp: Peri<'static, PB15>) {
    /*
     * TinyUSB's dwc2_clock_init() doesn't do anything on STM32. In order for host
     * to initialize, need to enable the USB 3.3V supply, peripheral clock, and
     * DM/DP alternate function
     */

    // Enable VDD33USB voltage detector so STM32's USB transceiver knows it is powered (wait for usb33rdy)
    cortex_m::interrupt::free(|_| {
        pac::PWR.cr3().modify(|w| {
            w.set_usb33den(true);
            w.set_usbregen(false);
        })
    });
    while !pac::PWR.cr3().read().usb33rdy() {}

    rcc::enable_and_reset::<USB_OTG_HS>();

    // Set up pins for PHY
    let af = AfType::output(OutputType::PushPull, Speed::VeryHigh);
    let mut dn = Flex::new(dn);
    let mut dp = Flex::new(dp);
    dn.set_as_af_unchecked(12, af);
    dp.set_as_af_unchecked(12, af);

    // Forget these so they don't go out of scope and get dropped, which would cause the pins to disconnect
    mem::forget(dn);
    mem::forget(dp);

    let host_init = tusb_rhport_init_t {
        role: tusb_role_t_TUSB_ROLE_HOST,
        speed: tusb_speed_t_TUSB_SPEED_FULL,
    };

    match unsafe { tusb_rhport_init(BOARD_TUH_RHPORT, &host_init) } {
        true => serial_log("USB MIDI Host initialized"),
        false => serial_error("ERROR: tusb_rhport_init failed. Unable to set up MIDI Host"),
    }
}

/// TinyUSB MIDI Host mount callback override
#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_mount_cb(idx: u8, mount_cb_data: *const tuh_midi_mount_cb_t) {
    let data = unsafe { &*mount_cb_data };
    MIDI_MOUNTED.store(true, Ordering::Relaxed);
    serial_log(&format!(
        "MIDI mounted: idx={} addr={} rx_cables={} tx_cables={}",
        idx, data.daddr, data.rx_cable_count, data.tx_cable_count,
    ));
}

/// TinyUSB MIDI Host munount callback override
#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_umount_cb(idx: u8) {
    MIDI_MOUNTED.store(false, Ordering::Relaxed);
    serial_log(&format!("MIDI unmounted: idx={}", idx));
}

/// TinyUSB MIDI Host rx callback override
/// Handles incoming MIDI events and passes off to application logic (does not currently pass through)
#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_rx_cb(idx: u8, xferred_bytes: u32) {
    if xferred_bytes == 0 {
        return;
    }

    let mut cable_num: u8 = 0;
    let mut buf: [u8; 48] = [0; 48];

    loop {
        let count = unsafe {
            tuh_midi_stream_read(idx, &mut cable_num, buf.as_mut_ptr(), buf.len() as u16)
        } as usize;

        if count == 0 {
            break;
        }

        // TODO: handle MIDI events
        let mut line = format!("MIDI cable {} rx:", cable_num);
        for byte in &buf[..count] {
            line.push_str(&format!(" {:02x}", byte));
        }
        serial_log(&line);
    }
}

// Required by TinyUSB
#[unsafe(no_mangle)]
pub static SystemCoreClock: u32 = 480_000_000;

#[unsafe(no_mangle)]
pub extern "C" fn tusb_time_millis_api() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}
