#![no_std]
#![no_main]

mod amp_envelope;
mod audio;
mod display;
mod effects;
mod engines;
mod logger;
mod midi;
mod oscillator;
mod parameter;
mod tinyusb;
mod utils;
mod voices;

use core::cell::RefCell;

use alloc::string::ToString;
use daisy_embassy::{
    default_rcc,
    hal::{bind_interrupts, peripherals},
    new_daisy_board,
};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_stm32::interrupt;
use embassy_stm32::{
    i2c::{Config, I2c},
    interrupt::{InterruptExt, Priority},
    usart,
};
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Timer;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use defmt_serial as _;

use crate::{
    audio::audio_handler,
    logger::{log_handler, serial_log},
    midi::{USB_IRQ_COUNT, initialize_midi_host, usb_host_task},
    tinyusb::{BOARD_TUH_RHPORT, tusb_int_handler},
};
use crate::{
    display::{DISPLAY, DisplayContent, display_handler},
    engines::fm::{ENGINE, FMSynth, voice_state_handler},
    voices::VOICES,
};

extern crate alloc;

/// Wraps `emballoc` so every allocation runs with interrupts disabled.
///
/// `emballoc` guards its heap with a `spin::Mutex`. On a single core with
/// interrupt priorities that deadlocks: if a lower-priority context is holding
/// the spinlock and a higher-priority interrupt preempts it and also allocates,
/// the ISR spins forever on a lock whose holder can never be scheduled again.
///
/// We allocate from thread mode (display, logger), from the P2 USB task, and
/// from the P1 OTG_HS ISR, so that race is very much reachable. A critical
/// section makes each allocation atomic, so the spinlock is never contended.
struct IrqSafeAlloc<const N: usize>(emballoc::Allocator<N>);

unsafe impl<const N: usize> core::alloc::GlobalAlloc for IrqSafeAlloc<N> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        cortex_m::interrupt::free(|_| unsafe { self.0.alloc(layout) })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        cortex_m::interrupt::free(|_| unsafe { self.0.dealloc(ptr, layout) })
    }
}

// 32K, up from 8K: serial_log allocates a String per line and emballoc is a
// simple free-list allocator, so heavy logging plus fragmentation could exhaust
// 8K. Lives in .bss in DTCMRAM (128K), so there's room.
#[global_allocator]
static ALLOCATOR: IrqSafeAlloc<32768> = IrqSafeAlloc(emballoc::Allocator::new());

/// Writes panics directly to USART1, bypassing the logger task, the channel,
/// and defmt entirely.
///
/// Previously `panic-probe` routed panics through `defmt` into `defmt-serial`,
/// which was never initialized — so a panic halted the firmware with no output
/// whatsoever, and looked exactly like "the log just stops".
/// Direct USART1 writer for fatal handlers. Bypasses the logger task, the
/// channel, and defmt, so it still works when everything else is wedged.
struct PanicUart;

impl core::fmt::Write for PanicUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            while !embassy_stm32::pac::USART1.isr().read().txe() {}
            embassy_stm32::pac::USART1
                .tdr()
                .write_value(embassy_stm32::pac::usart::regs::Dr(b as u32));
        }
        Ok(())
    }
}

/// Halts after letting the UART drain.
fn fatal_halt() -> ! {
    while !embassy_stm32::pac::USART1.isr().read().tc() {}
    loop {
        cortex_m::asm::nop();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;
    // We're already dead; take the UART regardless of who owns it.
    cortex_m::interrupt::disable();
    let _ = write!(PanicUart, "\r\n\r\n*** PANIC ***\r\n{}\r\n", info);
    fatal_halt()
}

/// Without this, a fault lands in cortex-m-rt's default handler — a bare
/// infinite loop with no output, which is indistinguishable from a hang.
#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    use core::fmt::Write;
    cortex_m::interrupt::disable();
    let _ = write!(
        PanicUart,
        "\r\n\r\n*** HARDFAULT ***\r\npc={:#010x} lr={:#010x} r0={:#010x} r1={:#010x} r2={:#010x} r3={:#010x}\r\n",
        ef.pc(),
        ef.lr(),
        ef.r0(),
        ef.r1(),
        ef.r2(),
        ef.r3(),
    );
    fatal_halt()
}

pub static SAMPLE_RATE: u32 = 44_100;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

static AUDIO_EXECUTOR: InterruptExecutor = InterruptExecutor::new();
static USB_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
fn USART3() {
    unsafe {
        AUDIO_EXECUTOR.on_interrupt();
    }
}

/// Borrows the UART4 *vector* only — the UART4 peripheral is never enabled, so
/// nothing else can pend it. Unrelated to the UART4 alternate function that
/// happens to exist on the display's pins.
#[interrupt]
fn UART4() {
    unsafe {
        USB_EXECUTOR.on_interrupt();
    }
}

#[interrupt]
fn OTG_HS() {
    USB_IRQ_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    unsafe {
        tusb_int_handler(BOARD_TUH_RHPORT, true);
    }
}

#[embassy_executor::main]
async fn main(low_priority_spawner: Spawner) {
    let peripherals = embassy_stm32::init(default_rcc());
    let board = new_daisy_board!(peripherals);

    // Start logger setup
    let logger: usart::UartTx<'_, embassy_stm32::mode::Blocking> =
        usart::UartTx::new_blocking(peripherals.USART1, board.pins.d13, usart::Config::default())
            .unwrap();
    low_priority_spawner.spawn(log_handler(logger).unwrap());

    serial_log("================================");
    // End logger setup

    // Start audio setup
    let audio_interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;
    let audio_interface = (audio_interface.start_interface().await).unwrap();
    let engine = ENGINE.init(Mutex::new(RefCell::new(FMSynth::new())));

    interrupt::USART3.set_priority(Priority::P0);
    let high_priority_executor = AUDIO_EXECUTOR.start(interrupt::USART3);
    high_priority_executor.spawn(audio_handler(audio_interface, engine).unwrap());
    high_priority_executor.spawn(voice_state_handler(engine).unwrap());

    serial_log("Audio Initialized");
    // End audio setup

    // Audio test
    low_priority_spawner.spawn(c_major().unwrap());

    // Start MIDI setup
    interrupt::OTG_HS.set_priority(Priority::P1);

    initialize_midi_host(board.pins.d29, board.pins.d30);

    // Pump TinyUSB on its own executor at P2: below audio's P0 so it can't
    // threaten the audio deadline, but above thread mode so the display's
    // blocking I2C flush (~90ms) and the blocking serial logger can't starve
    // enumeration, which has hard timing requirements.
    interrupt::UART4.set_priority(Priority::P2);
    let usb_executor = USB_EXECUTOR.start(interrupt::UART4);
    usb_executor.spawn(usb_host_task().unwrap());
    // End MIDI setup

    // Start display setup
    let i2c = I2c::new_blocking(
        peripherals.I2C1,
        board.pins.d11,
        board.pins.d12,
        Config::default(), // TODO: might want to make this slower
    );
    let display_interface = I2CDisplayInterface::new_custom_address(i2c, 0x3D);
    let mut display: Ssd1306<
        I2CInterface<I2c<'_, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
    > = Ssd1306::new(
        display_interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display.init().unwrap();

    low_priority_spawner.spawn(display_handler(display).unwrap());

    serial_log("Display Initialized");
    // End display setup
}

// TESTING STUFF
#[embassy_executor::task]
async fn c_major() {
    let mut receiver = VOICES.receiver().unwrap();
    let sender = VOICES.sender();

    let _value = receiver.get().await;
    let updated = [
        (true, 60),
        (false, 60),
        (false, 60),
        (false, 60),
        (false, 60),
    ];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C".to_string(),
        })
        .await;
    Timer::after_millis(500).await;

    let updated = [
        (true, 60),
        (true, 64),
        (false, 60),
        (false, 60),
        (false, 60),
    ];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C-E".to_string(),
        })
        .await;
    Timer::after_millis(500).await;

    let updated = [(true, 60), (true, 64), (true, 67), (false, 60), (false, 60)];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C-E-G".to_string(),
        })
        .await;
}
