//! MSP callbacks exported for access by the HAL

use crate::hal_sys::{self, gpio_pin, GPIO_InitTypeDef, SRAM_HandleTypeDef, TIM_HandleTypeDef};

// Copied constants because bindgen has trouble reading values with type casts or operations
const GPIO_AF2_TIM3: u32 = 2;
const GPIO_AF9_FMC: u32 = 9;
const GPIO_AF12_FMC: u32 = 12;
const GPIO_MODE_AF_PP: u32 = hal_sys::MODE_AF | hal_sys::OUTPUT_PP;
const GPIO_MODE_OUTPUT_PP: u32 = hal_sys::MODE_OUTPUT | hal_sys::OUTPUT_PP;
const RCC_PERIPHCLK_FMC: u32 = 0x01000000_u32;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_MspInit() {
    // Enable the SYSCFG clocks
    crate::STM_PERIPHERALS
        .as_mut()
        .unwrap()
        .RCC
        .apb4enr
        .modify(|_, w| w.syscfgen().set_bit());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_SRAM_MspInit(_hsram: *mut SRAM_HandleTypeDef) {
    let mut clk_init = hal_sys::RCC_PeriphCLKInitTypeDef::default();

    clk_init.PeriphClockSelection = RCC_PERIPHCLK_FMC.into();
    clk_init.FmcClockSelection = hal_sys::RCC_FMCCLKSOURCE_CLKP;
    assert!(
        hal_sys::HAL_RCCEx_PeriphCLKConfig(&raw mut clk_init) == hal_sys::HAL_StatusTypeDef_HAL_OK
    );

    // Enable the FMC clocks
    crate::STM_PERIPHERALS
        .as_mut()
        .unwrap()
        .RCC
        .ahb3enr
        .modify(|_, w| w.fmcen().set_bit());

    hal_sys::gpio_init(
        hal_sys::GPIOA_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF12_FMC,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: gpio_pin(4).into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOF_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF12_FMC,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: gpio_pin(12).into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOE_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF12_FMC,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: (gpio_pin(15)
                | gpio_pin(12)
                | gpio_pin(10)
                | gpio_pin(9)
                | gpio_pin(8)
                | gpio_pin(7))
            .into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOB_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF12_FMC,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: (gpio_pin(15) | gpio_pin(14)).into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOD_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF12_FMC,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: (gpio_pin(15)
                | gpio_pin(14)
                | gpio_pin(10)
                | gpio_pin(9)
                | gpio_pin(8)
                | gpio_pin(5)
                | gpio_pin(4)
                | gpio_pin(1)
                | gpio_pin(0))
            .into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOC_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_MEDIUM,
            Alternate: GPIO_AF9_FMC,
            Mode: GPIO_MODE_OUTPUT_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: gpio_pin(7).into(),
        },
    );

    hal_sys::HAL_GPIO_WritePin(
        hal_sys::GPIOC_BASE as *mut _,
        gpio_pin(7),
        hal_sys::GPIO_PinState_GPIO_PIN_SET,
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HAL_TIM_PWM_MspInit(htim: *mut TIM_HandleTypeDef) {
    if (*htim).Instance == hal_sys::TIM3_BASE as *mut _ {
        // Enable the TIM3 clocks
        crate::STM_PERIPHERALS
            .as_mut()
            .unwrap()
            .RCC
            .apb1lenr
            .modify(|_, w| w.tim3en().set_bit());
    }
}

/// Not exported, but used during init. Enables clocks for timers and modulators
pub unsafe fn timer_post_init(handle: &TIM_HandleTypeDef) {
    if handle.Instance != hal_sys::TIM3_BASE as *mut _ {
        return;
    }

    // Enable GPIOB & GPIOC clocks
    crate::STM_PERIPHERALS
        .as_mut()
        .unwrap()
        .RCC
        .ahb4enr
        .modify(|_, w| {
            w.gpioben().set_bit();
            w.gpiocen().set_bit()
        });

    hal_sys::gpio_init(
        hal_sys::GPIOB_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_LOW,
            Alternate: GPIO_AF2_TIM3,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: gpio_pin(1).into(),
        },
    );

    hal_sys::gpio_init(
        hal_sys::GPIOC_BASE,
        GPIO_InitTypeDef {
            Speed: hal_sys::GPIO_SPEED_FREQ_LOW,
            Alternate: GPIO_AF2_TIM3,
            Mode: GPIO_MODE_AF_PP,
            Pull: hal_sys::GPIO_NOPULL,
            Pin: gpio_pin(8).into(),
        },
    );
}
