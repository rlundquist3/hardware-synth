use alloc::format;
use core::sync::atomic::{AtomicU32, Ordering};
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
        BOARD_TUH_RHPORT, tuh_midi_mount_cb_t, tuh_midi_stream_read, tuh_task_ext,
        tusb_desc_configuration_t, tusb_desc_device_t, tusb_rhport_init, tusb_rhport_init_t,
        tusb_role_t_TUSB_ROLE_HOST, tusb_speed_t_TUSB_SPEED_FULL,
    },
};

pub mod notes;

/// Flip to `false` to silence the USB host diagnostics below.
const USB_DEBUG: bool = true;

/// Incremented by the OTG_HS ISR. Diagnostic only — distinguishes "no interrupts
/// at all" from "interrupts firing but enumeration never lands".
pub static USB_IRQ_COUNT: AtomicU32 = AtomicU32::new(0);

// Incremented by the local patches in hcd_dwc2.c when one of TinyUSB's
// unbounded spin-waits hits its guard. Nonzero means the dwc2 core wedged and
// upstream would have hung the whole system at interrupt priority.
#[unsafe(no_mangle)]
pub static TUSB_SPIN_CHANNEL_DISABLE: AtomicU32 = AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static TUSB_SPIN_IN_TOKEN: AtomicU32 = AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static TUSB_SPIN_RXFLVL: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
pub async fn usb_host_task() {
    let mut ticks: u32 = 0;
    let mut last_hprt: u32 = u32::MAX;

    loop {
        unsafe {
            tuh_task_ext(u32::MAX, false);
        }

        if USB_DEBUG {
            let hprt = pac::USB_OTG_HS.hprt().read();
            let gintsts = pac::USB_OTG_HS.gintsts().read();

            // Log on any port-state change so a transient connect can't be
            // missed between heartbeats, plus a heartbeat every ~500ms.
            ticks = ticks.wrapping_add(1);
            // Mask PLSTS (bits 11:10) — it's the live D+/D- line state and
            // toggles constantly during traffic, so comparing it makes every
            // sample look like a state change.
            let hprt_stable = hprt.0 & !(0b11 << 10);
            let changed = hprt_stable != last_hprt;
            if changed || ticks % 500 == 0 {
                last_hprt = hprt_stable;
                serial_log(&format!(
                    "{} hprt conn={} det={} ena={} pwr={} spd={} | gintsts={:#010x} host_mode={} | irqs={}",
                    if changed { "CHANGE" } else { "  ..  " },
                    hprt.pcsts() as u8,
                    hprt.pcdet() as u8,
                    hprt.pena() as u8,
                    hprt.ppwr() as u8,
                    hprt.pspd(),
                    gintsts.0,
                    gintsts.cmod() as u8,
                    USB_IRQ_COUNT.load(Ordering::Relaxed),
                ));

                let (dis, tok, rx) = (
                    TUSB_SPIN_CHANNEL_DISABLE.load(Ordering::Relaxed),
                    TUSB_SPIN_IN_TOKEN.load(Ordering::Relaxed),
                    TUSB_SPIN_RXFLVL.load(Ordering::Relaxed),
                );
                if dis | tok | rx != 0 {
                    serial_log(&format!(
                        "SPIN GUARD HIT: channel_disable={} in_token={} rxflvl={}",
                        dis, tok, rx
                    ));
                }

                // Port is up but the core has gone quiet: dump the host channel
                // state so we can tell "no transfer was ever submitted" from
                // "a transfer was submitted and the device never answered".
                if hprt.pcsts() && hprt.pena() {
                    let haint = pac::USB_OTG_HS.haint().read().haint();
                    let frame = pac::USB_OTG_HS.hfnum().read().frnum();
                    let mut line = format!("  ch: haint={:#06x} frame={}", haint, frame);
                    for ch in 0..4usize {
                        let cc = pac::USB_OTG_HS.hcchar(ch).read();
                        let ci = pac::USB_OTG_HS.hcint(ch).read();
                        if cc.chena() || ci.0 != 0 {
                            line.push_str(&format!(
                                " | ch{} ena={} dis={} ep={} dir={} hcint={:#06x}[{}{}{}{}{}{}{}]",
                                ch,
                                cc.chena() as u8,
                                cc.chdis() as u8,
                                cc.epnum(),
                                cc.epdir() as u8,
                                ci.0,
                                if ci.xfrc() { "XFRC " } else { "" },
                                if ci.stall() { "STALL " } else { "" },
                                if ci.nak() { "NAK " } else { "" },
                                if ci.ack() { "ACK " } else { "" },
                                if ci.txerr() { "TXERR " } else { "" },
                                if ci.bberr() { "BBERR " } else { "" },
                                if ci.dterr() { "DTERR " } else { "" },
                            ));
                        }
                    }
                    serial_log(&line);
                }
            }
        }

        // TinyUSB's own examples call tuh_task() in a tight loop. We can't do
        // that here — a busy-loop at P2 would starve thread mode (display,
        // logger) entirely — but 200us still pumps the enumeration state
        // machine 5x faster than a 1ms tick.
        Timer::after_micros(200).await;
    }
}

pub fn initialize_midi_host(dm: Peri<'static, PB14>, dp: Peri<'static, PB15>) {
    serial_log("Initializing MIDI Host...");

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

    if unsafe { tusb_rhport_init(BOARD_TUH_RHPORT, &host_init) } {
        serial_log("MIDI host initialized");
    } else {
        serial_log("ERROR: tusb_rhport_init failed");
    }
}

// ---- enumeration diagnostics (remove with USB_DEBUG) ----

/// Called for every host event queued by the HCD. Shows whether enumeration is
/// progressing at all, and how far it gets before the device drops.
#[unsafe(no_mangle)]
pub extern "C" fn tuh_event_hook_cb(rhport: u8, eventid: u32, in_isr: bool) {
    if !USB_DEBUG {
        return;
    }
    let name = match eventid {
        0 => "DEVICE_ATTACH",
        1 => "DEVICE_REMOVE",
        2 => "XFER_COMPLETE",
        3 => "FUNC_CALL",
        _ => "INVALID",
    };
    serial_log(&format!(
        "evt rhport={} {} (isr={})",
        rhport, name, in_isr as u8
    ));
}

/// Fires once the device descriptor has been read. If this appears, the control
/// endpoint works and GET_DESCRIPTOR succeeded.
#[unsafe(no_mangle)]
pub extern "C" fn tuh_enum_descriptor_device_cb(daddr: u8, desc_device: *const tusb_desc_device_t) {
    if !USB_DEBUG {
        return;
    }
    // USB descriptors are #[repr(packed)], so copy fields out by value —
    // taking references to them is unaligned and won't compile.
    let d = unsafe { core::ptr::read_unaligned(desc_device) };
    let (vid, pid) = (d.idVendor, d.idProduct);
    let (class, ep0) = (d.bDeviceClass, d.bMaxPacketSize0);
    serial_log(&format!(
        "enum daddr={} vid={:04x} pid={:04x} class={} ep0_size={}",
        daddr, vid, pid, class, ep0
    ));
}

/// Must return true or enumeration stops. Only here to log that we got this far.
#[unsafe(no_mangle)]
pub extern "C" fn tuh_enum_descriptor_configuration_cb(
    daddr: u8,
    cfg_index: u8,
    _desc_config: *const tusb_desc_configuration_t,
) -> bool {
    if USB_DEBUG {
        serial_log(&format!("enum daddr={} config #{}", daddr, cfg_index));
    }
    true
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
        let count = unsafe {
            tuh_midi_stream_read(idx, &mut cable_num, buf.as_mut_ptr(), buf.len() as u16)
        } as usize;

        if count == 0 {
            break;
        }

        let mut line = format!("MIDI cable {} rx:", cable_num);
        for byte in &buf[..count] {
            line.push_str(&format!(" {:02x}", byte));
        }
        serial_log(&line);
    }
}

// TODO: remove these
#[unsafe(no_mangle)]
pub extern "C" fn tuh_mount_cb(daddr: u8) {
    serial_log(&format!("USB device {} mounted", daddr));
}
#[unsafe(no_mangle)]
pub extern "C" fn tuh_umount_cb(daddr: u8) {
    serial_log(&format!("USB device {} unmounted", daddr));
}

#[unsafe(no_mangle)]
pub static SystemCoreClock: u32 = 480_000_000;

#[unsafe(no_mangle)]
pub extern "C" fn tusb_time_millis_api() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}
