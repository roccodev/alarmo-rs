core::include!(core::concat!(core::env!("OUT_DIR"), "/bindings.rs"));

/// Convenient wrapper for [`HAL_GPIO_Init`].
///
/// ## Safety
/// `gpio_base` must be one of the `GPIOx_BASE` constants.
pub unsafe fn gpio_init(gpio_base: u32, def: GPIO_InitTypeDef) {
    HAL_GPIO_Init(gpio_base as *mut _, &raw const def);
}

pub const fn gpio_pin(pin: u8) -> u16 {
    1 << pin
}
