//! Provides a display interface (see crate [`display_interface`]) to send data and commands
//! to the LCD on the Alarmo.

use core::cell::RefCell;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;
use stm32h7xx_hal::delay::Delay;
use stm32h7xx_hal::device::TIM3;
use stm32h7xx_hal::pwm::{ComplementaryImpossible, Pwm};
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
pub(crate) type ResetPin = Pin<'G', 4, Output<PushPull>>;

/// LCD peripheral.
///
/// ## Getting started
/// **Make sure to [`hard_reset`] the display!** Commands won't be accepted until you do.
/// You also need to set a **backlight** with [`set_backlight`] to actually get an image on the screen.
///
/// This struct implements a display interface according to the [`display_interface`] crate.
///
/// See the [LCD example] for a working implementation.
///
/// [`set_backlight`]: AlarmoDisplay::set_backlight
/// [`hard_reset`]: AlarmoDisplay::hard_reset
/// [LCD example]: https://github.com/roccodev/alarmo-rs/blob/master/examples/lcd.rs
pub struct AlarmoDisplay {
    backlight_timer: Pwm<TIM3, 3, ComplementaryImpossible>,
    select_pin: SelectPin,
    reset_pin: ResetPin,
    delay: &'static RefCell<Delay>,
}

pub struct HalDelay;

impl AlarmoDisplay {
    pub(crate) fn new(
        backlight_timer: Pwm<TIM3, 3, ComplementaryImpossible>,
        select_pin: SelectPin,
        reset_pin: Pin<'G', 4>,
        delay: &'static RefCell<Delay>,
    ) -> Self {
        Self {
            backlight_timer,
            select_pin,
            reset_pin: reset_pin.into_push_pull_output(),
            delay,
        }
    }

    /// Hard resets the display
    pub fn hard_reset(&mut self) {
        self.pin_select(false);

        self.reset_pin.set_low();
        self.reset_pin.set_high();

        self.delay.borrow_mut().delay_ms(120_u16);
    }

    pub fn set_backlight(&mut self, brightness: f32) {
        assert!(
            brightness <= 1.0 && brightness >= 0.0,
            "brightness [0.0, 1.0]"
        );
        self.backlight_timer
            .set_duty((self.backlight_timer.get_max_duty() as f32 * brightness) as u16);
        self.backlight_timer.enable();
    }

    fn pin_select(&mut self, select: bool) {
        if select {
            self.select_pin.set_low();
        } else {
            self.select_pin.set_high();
        }
    }
}

impl WriteOnlyDataCommand for AlarmoDisplay {
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
    fn delay_us(&mut self, us: u32) {
        self.delay_ms(us / 1_000);
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        use stm32h7xx_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
        unsafe { crate::DELAY.as_ref().unwrap().borrow_mut().delay_ms(ms) };
    }
}
