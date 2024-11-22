use embedded_hal::delay::DelayNs;
use stm32h7xx_hal::hal::blocking::delay::DelayUs;

pub struct HalDelay;

impl DelayNs for HalDelay {
    #[inline]
    fn delay_ns(&mut self, ns: u32) {
        self.delay_ms(ns / 1_000_000);
    }

    #[inline]
    fn delay_us(&mut self, us: u32) {
        self.delay_ms(us / 1_000);
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        use stm32h7xx_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
        unsafe { crate::DELAY.as_ref().unwrap().borrow_mut().delay_ms(ms) };
    }
}

impl DelayUs<u8> for HalDelay {
    fn delay_us(&mut self, us: u8) {
        <Self as DelayNs>::delay_us(self, us as u32);
    }
}
