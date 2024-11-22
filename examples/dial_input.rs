#![no_std]
#![no_main]

use alarmo::Alarmo;
use cortex_m_rt::entry;

// A panic handler is required
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut alarmo = unsafe { Alarmo::init() };

    // Turn the lights on
    alarmo.dial.lights_on();

    loop {
        // Set color based on dial rotation
        let rotation = alarmo.dial.rotation_deg();

        let (r, g, b) = hsv_to_rgb(rotation / 360f32, 1.0, 1.0);
        alarmo.dial.set_color(r, g, b);
    }
}

// Auxiliary code below

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h % 1.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - float_abs((h * 6.0) % 2.0 - 1.0));
    let m = v - c;

    let (r_prime, g_prime, b_prime) = match (h * 6.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    (r_prime + m, g_prime + m, b_prime + m)
}

fn float_abs(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & (i32::MAX as u32))
}
