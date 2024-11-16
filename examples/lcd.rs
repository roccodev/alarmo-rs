#![no_std]
#![no_main]

use alarmo::{display::HalDelay, Alarmo};
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
use panic_halt as _;

static mut ALARMO: Option<Alarmo> = None;

#[no_mangle]
pub unsafe fn main() {
    ALARMO = Some(Alarmo::init());

    // Initialize the display interface. This takes care of hard resetting the display.
    let disp_int = ALARMO.as_mut().unwrap().init_display();

    // Configure the display frontend library
    let mut delay = HalDelay;
    let mut disp = mipidsi::Builder::new(ST7789, disp_int)
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
    disp.dcs().di.set_backlight(1.0);

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
