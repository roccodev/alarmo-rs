#![no_std]
#![no_main]

use alarmo::{display::HalDelay, Alarmo};
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{RgbColor, *},
    text::Text,
};
use mipidsi::{
    models::ST7789,
    options::{ColorInversion, Orientation},
    TestImage,
};

// A panic handler is required
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut alarmo = unsafe { Alarmo::init() };

    // Hard reset the display
    alarmo.display.hard_reset();

    // Configure the display frontend library. This example shows `mipidsi`, but you can use
    // any crate compatible with `display_interface`
    let mut delay = HalDelay;
    let mut disp = mipidsi::Builder::new(ST7789, alarmo.display)
        // IMPORTANT! Alarmo LCD needs INVON
        .invert_colors(ColorInversion::Inverted)
        // IMPORTANT! The frame buffer is 240x320, for a horizontal picture it needs to be rotated
        .orientation(Orientation {
            mirrored: false,
            rotation: mipidsi::options::Rotation::Deg270,
        })
        // The `init` call blocks util the display is ready
        .init(&mut delay)
        .unwrap();

    // IMPORTANT! Display has no backlight by default, so you won't see anything unless you add one.
    // Getting back to the display interface is safe here as the backlight does not interfere with
    // the display state.
    unsafe { disp.dcs() }.di.set_backlight(1.0);

    // Display test image, see https://docs.rs/mipidsi/latest/mipidsi/struct.TestImage.html
    TestImage::new().draw(&mut disp).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::BLACK);
    Text::new("Rust on Alarmo!", Point::new(100, 100), style)
        .draw(&mut disp)
        .unwrap();

    loop {
        continue;
    }
}
