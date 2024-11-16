//! Provides a display interface (see crate [`display_interface`]) to send data and commands
//! to the LCD on the Alarmo.

use crate::hal_sys;
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;

const BASE: usize = 0xc0000000;
const RS_PIN: u8 = 6;
const COMMAND_PTR: *mut u8 = BASE as *mut u8;
const DATA8_PTR: *mut u8 = (BASE + (1 << (RS_PIN + 1))) as *mut u8;
const DATA16_PTR: *mut u16 = (BASE + (1 << (RS_PIN + 1))) as *mut u16;

pub struct AlarmoDisplayInterface<'a> {
    tim_handle: &'a mut hal_sys::TIM_HandleTypeDef,
}

pub struct HalDelay;

impl<'a> AlarmoDisplayInterface<'a> {
    pub(crate) unsafe fn init(
        tim_handle: &'a mut hal_sys::TIM_HandleTypeDef,
    ) -> AlarmoDisplayInterface<'a> {
        let display = AlarmoDisplayInterface { tim_handle };
        display.pin_select(false);

        // Hard reset the display
        hal_sys::HAL_GPIO_WritePin(
            hal_sys::GPIOG_BASE as *mut _,
            hal_sys::gpio_pin(4),
            hal_sys::GPIO_PinState_GPIO_PIN_RESET,
        );
        hal_sys::HAL_GPIO_WritePin(
            hal_sys::GPIOG_BASE as *mut _,
            hal_sys::gpio_pin(4),
            hal_sys::GPIO_PinState_GPIO_PIN_SET,
        );

        HalDelay.delay_ms(120);

        display
    }

    pub fn set_backlight(&mut self, brightness: f32) {
        assert!(
            brightness <= 1.0 && brightness >= 0.0,
            "brightness [0.0, 1.0]"
        );
        // TODO: doc safety
        unsafe {
            let period = (*self.tim_handle).Init.Period;
            (*(*self.tim_handle).Instance).CCR4 = (brightness * period as f32) as u32;
            hal_sys::HAL_TIM_PWM_Start(self.tim_handle as *mut _, hal_sys::TIM_CHANNEL_4);
        }
    }

    unsafe fn pin_select(&self, select: bool) {
        hal_sys::HAL_GPIO_WritePin(
            hal_sys::GPIOC_BASE as *mut _,
            hal_sys::gpio_pin(7),
            if select {
                hal_sys::GPIO_PinState_GPIO_PIN_RESET
            } else {
                hal_sys::GPIO_PinState_GPIO_PIN_SET
            },
        );
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
        unsafe { hal_sys::HAL_Delay(ns / 1_000_000) };
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        unsafe { hal_sys::HAL_Delay(ms) };
    }

    #[inline]
    fn delay_us(&mut self, us: u32) {
        self.delay_ms(us / 1_000);
    }
}
