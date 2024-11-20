//! You can enable the `panic` feature to get panic messages to display on the Alarmo's screen,
//! instead of silently halting.

#![no_main]
#![no_std]

use alarmo::input::{Button, InterruptMode};
use alarmo::Alarmo;
use cortex_m_rt::{entry, exception};

#[entry]
fn main() -> ! {
    let mut alarmo = unsafe { Alarmo::init() };

    // Panic when a button is pressed
    alarmo.buttons.into_interrupts(
        &mut alarmo.ext_interrupts,
        InterruptMode::Dial,
        |button, _| {
            match button {
                Button::DialClick => panic!("Dial click!"),
                // This triggers a HardFault
                Button::Back => cortex_m::asm::udf(),
                _ => {}
            }
        },
    );

    loop {}
}

// Some other exceptions to intercept

#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    // Note: enable the `alloc` feature to see the message, otherwise messages with placeholders
    // will appear empty
    panic!("HardFault at {:#?}", ef)
}

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
