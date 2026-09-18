pub mod tasks;

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
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};

use logger::{serial_error, serial_log};
use rust_tinyusb_host::{
    tuh_deinit, tuh_midi_mount_cb_t, tuh_midi_stream_read, tuh_task_ext, tusb_rhport_init,
    tusb_rhport_init_t, tusb_role_t_TUSB_ROLE_HOST, tusb_speed_t_TUSB_SPEED_FULL,
};
use synth_core::midi::MidiMessage;

pub static MIDI_BUFFER: Channel<CriticalSectionRawMutex, MidiMessage, 16> = Channel::new();

static MIDI_MOUNTED: AtomicBool = AtomicBool::new(false);

// USB-A Port on pins D29 (D-) / D30 (D+) (onboard USB-C is port 0)
pub const BOARD_TUH_RHPORT: u8 = 1;

const MAX_RECOVERY_ATTEMPTS: u8 = 3;

// How long an attached device may fail to enumerate before the host is reset.
const ENUMERATION_TIMEOUT: Duration = Duration::from_secs(3);

// How long the port may sit connected-but-disabled (dwc2 babble) before reset.
const PORT_DEAD_TIMEOUT: Duration = Duration::from_millis(10);

// How long to yield to lower-priority tasks between host polls.
const POLL_INTERVAL: Duration = Duration::from_micros(200);

const HOST_INIT: tusb_rhport_init_t = tusb_rhport_init_t {
    role: tusb_role_t_TUSB_ROLE_HOST,
    speed: tusb_speed_t_TUSB_SPEED_FULL,
};

#[derive(PartialEq, Eq, Clone, Copy)]
struct PortState {
    connected: bool,
    enabled: bool,
    powered: bool,
    overcurrent: bool,
    speed: u8,
}

// HPRT.PLSTS with both single-ended lines high: not a legal USB signaling state.
const ERROR_LINE_STATE: u8 = 0b11;

#[embassy_executor::task]
pub async fn usb_host_task() {
    let mut stalled_since: Option<Instant> = None;
    let mut port_dead_since: Option<Instant> = None;
    let mut attempts: u8 = 0;
    let mut se1_reported = false;

    loop {
        unsafe {
            tuh_task_ext(u32::MAX, false);
        }

        /*
         * If a controller was plugged in at boot and attempted to connect before
         * the host started, it won't be mounted. Wait ~3s and reset bus to mount
         * controllers in these cases.
         * Only hit if something is plugged in, limited retry attempts.
         */
        let hprt = pac::USB_OTG_HS.hprt().read();
        let port = PortState {
            connected: hprt.pcsts(),
            enabled: hprt.pena(),
            powered: hprt.ppwr(),
            overcurrent: hprt.poca(),
            speed: hprt.pspd(),
        };
        let mounted = MIDI_MOUNTED.load(Ordering::Relaxed);

        let line_state = hprt.plsts();

        /*
         * SE1 means D+ and D- are both high, which USB does not define. A short
         * between the pair reads this way: the device's 1.5k pullup against both
         * host pulldowns sits around 2.75V, above the receiver threshold on both
         * lines. Reported once per attach -- the line is sampled asynchronously
         * to bus traffic, so a single reading is not worth acting on.
         */
        if port.connected && !mounted && line_state == ERROR_LINE_STATE && !se1_reported {
            se1_reported = true;
            serial_error("bus in SE1 (D+ and D- both high) -- check for a short across the pair");
        } else if !port.connected {
            se1_reported = false;
        }

        /*
         * dwc2 disables the port if there is a babble error, which is not
         * handled by TinyUSB. Attempt to reconnect if this happens.
         *
         * TODO: frequency of this error is likely related to quality of
         * USB port wiring. Just let it ride for now, but something to keep
         * an eye on when moving hardware past the prototype stage.
         */
        if mounted && port.connected && !port.enabled {
            if port_dead_since.get_or_insert_with(Instant::now).elapsed() >= PORT_DEAD_TIMEOUT {
                port_dead_since = None;
                serial_error("USB port disabled, reinitializing host");
                reinitialize_host().await;
            }
        } else {
            port_dead_since = None;
        }

        if !port.connected || mounted {
            stalled_since = None;
            attempts = 0;
        } else if stalled_since.get_or_insert_with(Instant::now).elapsed() >= ENUMERATION_TIMEOUT {
            stalled_since = None;
            if attempts < MAX_RECOVERY_ATTEMPTS {
                attempts += 1;
                serial_log("USB enumeration stalled, reinitializing host");
                reinitialize_host().await;
            } else if attempts == MAX_RECOVERY_ATTEMPTS {
                attempts += 1;
                serial_error("device attached but will not enumerate");
            }
        }

        // Timeout to yield to lower-priority tasks
        Timer::after(POLL_INTERVAL).await;
    }
}

/// Tears down and reinitializes USB Host, forcing re-enumeration
async fn reinitialize_host() {
    unsafe {
        tuh_deinit(BOARD_TUH_RHPORT);
    }
    MIDI_MOUNTED.store(false, Ordering::Relaxed);
    Timer::after_millis(50).await;

    match unsafe { tusb_rhport_init(BOARD_TUH_RHPORT, &HOST_INIT) } {
        true => serial_log("USB host reinit successful"),
        false => serial_error("USB host reinit failed"),
    };
}

pub fn initialize_midi_host(dn: Peri<'static, PB14>, dp: Peri<'static, PB15>) {
    /*
     * TinyUSB's dwc2_clock_init() doesn't do anything on STM32. In order for host
     * to initialize, need to enable the USB 3.3V supply, peripheral clock, and
     * USB data pin alternate functions
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

    match unsafe { tusb_rhport_init(BOARD_TUH_RHPORT, &HOST_INIT) } {
        true => serial_log("USB MIDI Host initialized"),
        false => serial_error("tusb_rhport_init failed. Unable to set up MIDI Host"),
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

        // TODO: handle additional bytes?
        let midi_message = MidiMessage(buf[0], buf[1], buf[2]);
        // TODO: dial in buffer size and figure out a way to elegantly handle errors
        match MIDI_BUFFER.try_send(midi_message) {
            Ok(()) => {}
            Err(_) => serial_error("MIDI buffer full"),
        };

        // TODO: handle MIDI events
        let mut line = format!("MIDI cable {} rx:", cable_num);
        for byte in &buf[..count] {
            line.push_str(&format!(" {}", byte));
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
