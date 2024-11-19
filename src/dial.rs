//! Dial peripheral, used for input and LED

use crate::pac::timers::DialTimers;
use stm32h7xx_hal::prelude::_embedded_hal_PwmPin;

/// Dial peripheral, found at the top of the device.
///
/// This interface currently allows controlling the dial LED.
///
/// See the [Dial LED example] for a working implementation.
///
/// [Dial LED example]: https://github.com/roccodev/alarmo-rs/blob/master/examples/dial_led.rs
pub struct Dial {
    pub(crate) timers: DialTimers,
}

impl Dial {
    /// Turns the lights on, making sure the respective timers are running.
    pub fn lights_on(&mut self) {
        self.timers.tim1_ch1.enable();
        self.timers.tim1_ch3.enable();
        self.timers.tim3_ch3.enable();
    }

    /// Changes the color of the dial LEDs.
    ///
    /// ## Params
    /// `0.0 <= r, g, b <= 1.0`
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
        self.timers
            .tim1_ch3
            .set_duty((self.timers.tim1_ch1.get_max_duty() as f32 * r) as u16);
        self.timers
            .tim1_ch1
            .set_duty((self.timers.tim1_ch1.get_max_duty() as f32 * g) as u16);
        self.timers
            .tim3_ch3
            .set_duty((self.timers.tim1_ch1.get_max_duty() as f32 * b) as u16);
    }

    /// Turns the lights off, but keeps the timers running.
    pub fn lights_off(&mut self) {
        self.set_color(0.0, 0.0, 0.0);
    }
}
