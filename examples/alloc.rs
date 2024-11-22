//! Heap allocation example

#![no_std]
#![no_main]

// Also enable the "alloc" feature in alarmo
extern crate alloc;
use alloc::string::ToString;
use alloc::vec;

use alarmo::delay::HalDelay;
use alarmo::display::AlarmoDisplay;
use alarmo::Alarmo;

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation};
use mipidsi::{Display, NoResetPin};

// A panic handler is required
use panic_halt as _;

struct TestDisplay(Display<AlarmoDisplay, ST7789, NoResetPin>);

#[entry]
fn main() -> ! {
    let alarmo = unsafe { Alarmo::init() };

    let mut test_vec = vec![0usize; 1];
    let mut display = TestDisplay::new(alarmo.display);

    loop {
        // Count from 1 to 100, printing all numbers.
        if test_vec.len() < 100 {
            test_vec.push(test_vec.len());
            display.write(&test_vec);
            alarmo.delay.borrow_mut().delay_ms(1000_u16);
        }
    }
}

impl TestDisplay {
    fn new(mut display: AlarmoDisplay) -> Self {
        display.hard_reset();

        let mut delay = HalDelay;
        let mut disp = mipidsi::Builder::new(ST7789, display)
            .invert_colors(ColorInversion::Inverted)
            .orientation(Orientation {
                mirrored: false,
                rotation: mipidsi::options::Rotation::Deg270,
            })
            .init(&mut delay)
            .unwrap();
        unsafe { disp.dcs() }.di.set_backlight(1.0);
        disp.clear(Rgb565::WHITE).unwrap();
        Self(disp)
    }

    fn write(&mut self, nums: &[usize]) {
        self.0.clear(Rgb565::WHITE).unwrap();
        let (mut x, mut y) = (10, 10);
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::BLACK);
        for num in nums {
            Text::new(&num.to_string(), Point::new(x, y), style)
                .draw(&mut self.0)
                .unwrap();
            x += 30;
            if x > 300 {
                x = 0;
                y += 30;
            }
        }
    }
}
