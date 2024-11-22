//! Dial peripheral, used for input and LED

use crate::pac::adc::AdcBuffers;
use crate::pac::timers::DialTimers;
use core::cell::UnsafeCell;
use cortex_m::peripheral::SCB;
use micromath::F32Ext;
use stm32h7xx_hal::prelude::_embedded_hal_PwmPin;

/// Dial peripheral, found at the top of the device.
///
/// This interface currently allows controlling the dial LED.
///
/// See the [Dial LED example] for a working implementation.
///
/// [Dial LED example]: https://github.com/roccodev/alarmo-rs/blob/master/examples/dial_led.rs
pub struct Dial {
    timers: DialTimers,
    adc_buffers: AdcBuffers,
    scb: UnsafeCell<SCB>,
}

impl Dial {
    pub(crate) fn new(timers: DialTimers, adc_buffers: AdcBuffers, scb: SCB) -> Self {
        Self {
            timers,
            adc_buffers,
            scb: UnsafeCell::new(scb),
        }
    }

    /// Returns the last known rotation of the dial, in radians (`(-pi, pi]`)
    pub fn rotation_rad(&self) -> f32 {
        unsafe {
            // The cache functions do not actually use self
            let scb = &mut *self.scb.get();
            scb.invalidate_dcache_by_address(self.adc_buffers.adc1 as usize, 32);
            scb.invalidate_dcache_by_address(self.adc_buffers.adc2 as usize, 32);
        }
        let (y, x) = unsafe {
            (
                self.adc_buffers.adc1.read_volatile(),
                self.adc_buffers.adc2.read_volatile(),
            )
        };
        let (y, x) = (y as f32 / 65535f32, x as f32 / 65535f32);
        f32::atan2(y - 0.5, x - 0.5)
    }

    /// Returns the last known rotation of the dial, in normalized degrees (`[0, 360)`)
    pub fn rotation_deg(&self) -> f32 {
        let mut rot = self.rotation_rad().to_degrees();
        if rot < 0.0 {
            rot = (360f32 - ((-rot) % 360f32)) % 360f32;
        }
        rot
    }

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
