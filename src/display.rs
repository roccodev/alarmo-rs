//! Provides a display interface (see crate [`display_interface`]) to send data and commands
//! to the LCD on the Alarmo.

use crate::{pac::timers::Timers, ALARMO};
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;
use stm32h7xx_hal::{
    gpio::{Output, Pin, PushPull},
    prelude::_embedded_hal_PwmPin,
};

const BASE: usize = 0xc0000000;
const RS_PIN: u8 = 6;
const COMMAND_PTR: *mut u8 = BASE as *mut u8;
const DATA8_PTR: *mut u8 = (BASE + (1 << (RS_PIN + 1))) as *mut u8;
const DATA16_PTR: *mut u16 = (BASE + (1 << (RS_PIN + 1))) as *mut u16;

pub(crate) type SelectPin = Pin<'C', 7, Output<PushPull>>;

pub struct AlarmoDisplayInterface<'a> {
    timers: &'a mut Timers,
    select_pin: SelectPin,
}

pub struct HalDelay;

impl<'a> AlarmoDisplayInterface<'a> {
    pub(crate) fn init(
        timers: &'a mut Timers,
        reset_pin: Pin<'G', 4>,
        select_pin: SelectPin,
    ) -> AlarmoDisplayInterface<'a> {
        let mut display = AlarmoDisplayInterface { timers, select_pin };
        display.pin_select(false);

        // Hard reset the display
        let mut reset_pin = reset_pin.into_push_pull_output();
        reset_pin.set_low();
        reset_pin.set_high();

        HalDelay.delay_ms(120);

        display
    }

    pub fn set_backlight(&mut self, brightness: f32) {
        assert!(
            brightness <= 1.0 && brightness >= 0.0,
            "brightness [0.0, 1.0]"
        );
        self.timers
            .tim3_ch4
            .set_duty((self.timers.tim3_ch4.get_max_duty() as f32 * brightness) as u16);
        self.timers.tim3_ch4.enable();
    }

    fn pin_select(&mut self, select: bool) {
        if select {
            self.select_pin.set_low();
        } else {
            self.select_pin.set_high();
        }
    }
}

impl<'a> WriteOnlyDataCommand for AlarmoDisplayInterface<'a> {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        let DataFormat::U8(&[byte]) = cmd else {
            return Err(DisplayError::InvalidFormatError);
        };
        unsafe {
            self.pin_select(true);
            COMMAND_PTR.write_volatile(byte);
        }
        Ok(())
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        unsafe {
            match buf {
                DataFormat::U8(bytes) => {
                    for b in bytes {
                        DATA8_PTR.write_volatile(*b);
                    }
                }
                DataFormat::U16(bytes) => {
                    for short in bytes {
                        DATA16_PTR.write_volatile(*short);
                    }
                }
                DataFormat::U16BEIter(iter) => {
                    for short in iter {
                        DATA16_PTR.write_volatile(short);
                    }
                }
                _ => return Err(DisplayError::InvalidFormatError),
            }
            self.pin_select(false);
        }
        Ok(())
    }
}

impl DelayNs for HalDelay {
    #[inline]
    fn delay_ns(&mut self, ns: u32) {
        self.delay_ms(ns / 1_000_000);
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        use stm32h7xx_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
        unsafe { ALARMO.as_mut().unwrap().delay.delay_ms(ms) };
    }

    #[inline]
    fn delay_us(&mut self, us: u32) {
        self.delay_ms(us / 1_000);
    }
}
