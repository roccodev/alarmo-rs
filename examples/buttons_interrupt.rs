//! Interrupt-based version of the buttons example

#![no_std]
#![no_main]

use alarmo::dial::Dial;
use alarmo::input::{Button, InterruptMode};
use alarmo::Alarmo;
use core::cell::RefCell;
use core::sync::atomic::{AtomicIsize, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

// Panic handler is required
use panic_halt as _;

static DIAL: Mutex<RefCell<Option<Dial>>> = Mutex::new(RefCell::new(None));
static COLOR_INDEX: AtomicIsize = AtomicIsize::new(0);
static COLORS: [(f32, f32, f32); 4] = [
    (1.0, 1.0, 1.0),
    (1.0, 0.0, 0.0),
    (0.0, 1.0, 0.0),
    (0.0, 0.0, 1.0),
];

#[entry]
fn main() -> ! {
    let mut alarmo = unsafe { Alarmo::init() };

    // Turn the lights on
    alarmo.dial.set_color(COLORS[0].0, COLORS[0].1, COLORS[0].2);
    alarmo.dial.lights_on();

    cortex_m::interrupt::free(|cs| {
        DIAL.borrow(cs).replace(Some(alarmo.dial));
    });

    // For interrupt-based input, you can't enable both the mail and dial click buttons
    alarmo.buttons.into_interrupts(
        &mut alarmo.ext_interrupts,
        InterruptMode::Dial,
        |button, cs| {
            let mut dial = DIAL.borrow(cs).borrow_mut();
            let dial = dial.as_mut().unwrap();
            let next_color = match button {
                Button::Back => {
                    // Back button: previous color
                    let index = COLOR_INDEX
                        .load(Ordering::Relaxed)
                        .wrapping_sub(1)
                        .rem_euclid(COLORS.len() as isize);
                    COLOR_INDEX.store(index, Ordering::Relaxed);
                    COLORS[usize::try_from(index).unwrap()]
                }
                Button::DialClick => {
                    // Dial click: next color
                    let index = COLOR_INDEX
                        .load(Ordering::Relaxed)
                        .wrapping_add(1)
                        .rem_euclid(COLORS.len() as isize);
                    COLOR_INDEX.store(index, Ordering::Relaxed);
                    COLORS[usize::try_from(index).unwrap()]
                }
                // Disabled, see above
                Button::Mail => unreachable!(),
            };
            dial.set_color(next_color.0, next_color.1, next_color.2);
        },
    );

    loop {}
}
