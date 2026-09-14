use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use embassy_stm32::{
    Peri,
    gpio::{AfType, Flex, OutputType, Speed},
    pac,
    peripherals::{PB14, PB15, USB_OTG_HS},
    rcc,
};
use embassy_time::Timer;

use crate::{
    logger::serial_log,
    tinyusb::{
        BOARD_TUH_RHPORT, tuh_midi_mount_cb_t, tuh_midi_stream_read, tuh_rhport_reset_bus,
        tuh_task_ext, tusb_rhport_init, tusb_rhport_init_t, tusb_role_t_TUSB_ROLE_HOST,
        tusb_speed_t_TUSB_SPEED_FULL,
    },
};

pub mod notes;

/// Set while a MIDI interface is mounted. Used to tell a healthy idle port from
/// one that came up but never enumerated.
static MIDI_MOUNTED: AtomicBool = AtomicBool::new(false);

/// Incremented by the guards in hcd_dwc2.c when one of TinyUSB's unbounded
/// spin-waits hits its iteration cap. Nonzero means the dwc2 core wedged.
#[unsafe(no_mangle)]
pub static TUSB_SPIN_CHANNEL_DISABLE: AtomicU32 = AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static TUSB_SPIN_IN_TOKEN: AtomicU32 = AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static TUSB_SPIN_RXFLVL: AtomicU32 = AtomicU32::new(0);

/// How long the port may sit connected and enabled without a MIDI interface
/// mounting before we pulse a bus reset. Enumeration normally completes in well
/// under a second.
const ENUM_TIMEOUT_TICKS: u32 = 15_000; // ~3s at 200us per tick

#[embassy_executor::task]
pub async fn usb_host_task() {
    let mut stalled_ticks: u32 = 0;

    loop {
        unsafe {
            tuh_task_ext(u32::MAX, false);
        }

        // A device that is already plugged in at boot is powered and asserting
        // its pull-up before the host starts, so TinyUSB sees an immediate
        // attach rather than a clean plug event and enumeration sometimes never
        // takes. Pulsing a bus reset does what replugging the cable does.
        let hprt = pac::USB_OTG_HS.hprt().read();
        if hprt.pcsts() && hprt.pena() && !MIDI_MOUNTED.load(Ordering::Relaxed) {
            stalled_ticks += 1;
            if stalled_ticks >= ENUM_TIMEOUT_TICKS {
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

        // TinyUSB's own examples call tuh_task() in a tight loop. We can't do
        // that here — a busy-loop at P2 would starve thread mode — but 200us
        // still pumps the enumeration state machine promptly.
        Timer::after_micros(200).await;
    }
}

pub fn initialize_midi_host(dm: Peri<'static, PB14>, dp: Peri<'static, PB15>) {
    // TinyUSB's dwc2_clock_init() is a no-op on STM32: enabling the peripheral
    // clock, the USB 3.3V supply, and the DM/DP alternate function is the board
    // layer's job. Without it dwc2_core_init() fails its check_dwc2() assert.

    // VDD33USB is fed directly from the board's 3.3V rail, so enable the voltage
    // detector and leave the internal regulator off.
    cortex_m::interrupt::free(|_| {
        pac::PWR.cr3().modify(|w| {
            w.set_usb33den(true);
            w.set_usbregen(false);
        })
    });
    while !pac::PWR.cr3().read().usb33rdy() {}

    rcc::enable_and_reset::<USB_OTG_HS>();

    // PB14 = OTG_HS_DM, PB15 = OTG_HS_DP, both AF12.
    // Forget the Flex handles — their Drop would set the pins back to disconnected.
    let af = AfType::output(OutputType::PushPull, Speed::VeryHigh);
    let mut dm = Flex::new(dm);
    let mut dp = Flex::new(dp);
    dm.set_as_af_unchecked(12, af);
    dp.set_as_af_unchecked(12, af);
    core::mem::forget(dm);
    core::mem::forget(dp);

    let host_init = tusb_rhport_init_t {
        role: tusb_role_t_TUSB_ROLE_HOST,
        speed: tusb_speed_t_TUSB_SPEED_FULL,
    };

    if !unsafe { tusb_rhport_init(BOARD_TUH_RHPORT, &host_init) } {
        serial_log("ERROR: tusb_rhport_init failed");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_mount_cb(idx: u8, mount_cb_data: *const tuh_midi_mount_cb_t) {
    let data = unsafe { &*mount_cb_data };
    MIDI_MOUNTED.store(true, Ordering::Relaxed);
    serial_log(&format!(
        "MIDI mounted: idx={} addr={} rx_cables={} tx_cables={}",
        idx, data.daddr, data.rx_cable_count, data.tx_cable_count,
    ));
}

#[unsafe(no_mangle)]
pub extern "C" fn tuh_midi_umount_cb(idx: u8) {
    MIDI_MOUNTED.store(false, Ordering::Relaxed);
    serial_log(&format!("MIDI unmounted: idx={}", idx));
}

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

        // TODO: parse into note on/off and drive VOICES instead of logging.
        let mut line = format!("MIDI cable {} rx:", cable_num);
        for byte in &buf[..count] {
            line.push_str(&format!(" {:02x}", byte));
        }
        serial_log(&line);
    }
}

#[unsafe(no_mangle)]
pub static SystemCoreClock: u32 = 480_000_000;

#[unsafe(no_mangle)]
pub extern "C" fn tusb_time_millis_api() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}
