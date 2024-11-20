#![no_std]
#![no_main]

use alarmo::Alarmo;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let mut alarmo = unsafe { Alarmo::init() };

    let colors = [
        (1.0, 1.0, 1.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (0.0, 0.0, 1.0),
    ];
    let mut color = 0isize;
    let mut off = false;

    alarmo.dial.set_color(
        colors[color as usize].0,
        colors[color as usize].1,
        colors[color as usize].2,
    );
    alarmo.dial.lights_on();

    let mut last_press = [0, 0, 0];
    let mut cycle = 0;
    let debounce = 200_000;

    loop {
        if alarmo.buttons.dial_click() && cycle - last_press[0] > debounce {
            panic!("Dial click");
            // Toggle
            if off {
                alarmo.dial.set_color(
                    colors[color as usize].0,
                    colors[color as usize].1,
                    colors[color as usize].2,
                );
            } else {
                alarmo.dial.set_color(0.0, 0.0, 0.0);
            }
            off = !off;
            last_press[0] = cycle;
        }
        if alarmo.buttons.mail() && cycle - last_press[1] > debounce {
            // Next color
            color = color.wrapping_sub(1).rem_euclid(colors.len() as isize);
            alarmo.dial.set_color(
                colors[color as usize].0,
                colors[color as usize].1,
                colors[color as usize].2,
            );
            last_press[1] = cycle;
        }
        if alarmo.buttons.back() && cycle - last_press[2] > debounce {
            // Previous color
            color = color.wrapping_add(1) % colors.len() as isize;
            alarmo.dial.set_color(
                colors[color as usize].0,
                colors[color as usize].1,
                colors[color as usize].2,
            );
            last_press[2] = cycle;
        }
        cycle += 1;
    }
}
