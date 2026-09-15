use core::fmt::Write;
use embassy_stm32::pac::{USART1, usart};

/**
 * Writes panics to USART1 (which is used for logging in the healthy application)
 */
struct PanicUart;

impl core::fmt::Write for PanicUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            while !USART1.isr().read().txe() {}
            USART1.tdr().write_value(usart::regs::Dr(b as u32));
        }
        Ok(())
    }
}

// Halt after letting the UART drain
fn fatal_halt() -> ! {
    while !USART1.isr().read().tc() {}
    loop {
        cortex_m::asm::nop();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    let _ = write!(PanicUart, "\r\n\r\n====== PANIC ======\r\n{}\r\n", info);
    fatal_halt()
}

// Log what would've been swallowed by cortex-m-rt's default handler
#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    cortex_m::interrupt::disable();
    let _ = write!(
        PanicUart,
        "\r\n\r\n====== HARDFAULT ======\r\npc={:#010x} lr={:#010x} r0={:#010x} r1={:#010x} r2={:#010x} r3={:#010x}\r\n",
        ef.pc(),
        ef.lr(),
        ef.r0(),
        ef.r1(),
        ef.r2(),
        ef.r3(),
    );
    fatal_halt()
}
