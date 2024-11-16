#![no_std]

use hal_sys::{
    FMC_NORSRAM_TimingTypeDef, SRAM_HandleTypeDef, TIM_HandleTypeDef, TIM_MasterConfigTypeDef,
    TIM_OC_InitTypeDef,
};
use stm32h7xx_hal::pac::Peripherals;

mod arch;
mod hal_msp;
#[allow(warnings)]
pub mod hal_sys;
mod interrupt_handlers;

pub(crate) static mut STM_PERIPHERALS: Option<Peripherals> = None;

extern "C" {
    pub fn GaryMain(tim_handle: *const TIM_HandleTypeDef) -> u32;
}

pub struct Alarmo {
    sram_handle: SRAM_HandleTypeDef,
    pub tim3_handle: TIM_HandleTypeDef,
}

impl Alarmo {
    pub unsafe fn init() -> Alarmo {
        let mut cortex = cortex_m::Peripherals::take().unwrap();
        STM_PERIPHERALS = Peripherals::take();

        arch::enable_instruction_cache(&mut cortex);
        arch::enable_data_cache(&mut cortex);
        arch::enable_interrupts();
        hal_sys::HAL_Init();

        let sram_handle = init_sram();
        let tim3_handle = init_lcd_timers();

        Alarmo {
            sram_handle,
            tim3_handle,
        }
    }
}

unsafe fn init_sram() -> SRAM_HandleTypeDef {
    let mut sram_handle = SRAM_HandleTypeDef::default();
    sram_handle.Instance = hal_sys::FMC_Bank1_R_BASE as *mut _;
    sram_handle.Extended = hal_sys::FMC_Bank1E_R_BASE as *mut _;
    sram_handle.Init.WaitSignalActive = 0;
    sram_handle.Init.WriteOperation = hal_sys::FMC_WRITE_OPERATION_ENABLE;
    sram_handle.Init.NSBank = hal_sys::FMC_NORSRAM_BANK1;
    sram_handle.Init.MemoryDataWidth = hal_sys::FMC_NORSRAM_MEM_BUS_WIDTH_16;
    sram_handle.Init.BurstAccessMode = hal_sys::FMC_BURST_ACCESS_MODE_DISABLE;
    sram_handle.Init.DataAddressMux = hal_sys::FMC_DATA_ADDRESS_MUX_DISABLE;
    sram_handle.Init.MemoryType = hal_sys::FMC_MEMORY_TYPE_SRAM;
    sram_handle.Init.WaitSignalPolarity = hal_sys::FMC_WAIT_SIGNAL_POLARITY_LOW;

    let mut timing = FMC_NORSRAM_TimingTypeDef::default();
    timing.BusTurnAroundDuration = 0;
    timing.CLKDivision = 1;
    timing.DataLatency = 0;
    timing.AccessMode = hal_sys::FMC_ACCESS_MODE_A;
    timing.DataSetupTime = 2;
    timing.AddressSetupTime = 2;
    timing.AddressHoldTime = 0;

    assert!(
        hal_sys::HAL_SRAM_Init(&raw mut sram_handle, &raw mut timing, core::ptr::null_mut())
            == hal_sys::HAL_StatusTypeDef_HAL_OK
    );
    hal_sys::HAL_SetFMCMemorySwappingConfig(hal_sys::FMC_SWAPBMAP_SDRAM_SRAM);
    sram_handle
}

unsafe fn init_lcd_timers() -> TIM_HandleTypeDef {
    let mut handle = TIM_HandleTypeDef::default();
    handle.Instance = hal_sys::TIM3_BASE as *mut _;
    handle.Init.AutoReloadPreload = hal_sys::TIM_AUTORELOAD_PRELOAD_DISABLE;
    handle.Init.Prescaler = 0;
    handle.Init.CounterMode = hal_sys::TIM_COUNTERMODE_UP;
    handle.Init.Period = u16::MAX.into();
    handle.Init.ClockDivision = hal_sys::TIM_CLOCKDIVISION_DIV1;
    assert!(hal_sys::HAL_TIM_PWM_Init(&raw mut handle) == hal_sys::HAL_StatusTypeDef_HAL_OK);

    let mut master_cfg = TIM_MasterConfigTypeDef::default();
    master_cfg.MasterSlaveMode = hal_sys::TIM_MASTERSLAVEMODE_DISABLE;
    master_cfg.MasterOutputTrigger = hal_sys::TIM_TRGO_RESET;
    assert!(
        hal_sys::HAL_TIMEx_MasterConfigSynchronization(&raw mut handle, &raw const master_cfg)
            == hal_sys::HAL_StatusTypeDef_HAL_OK
    );

    let mut channel_cfg = TIM_OC_InitTypeDef::default();
    channel_cfg.OCFastMode = hal_sys::TIM_OCFAST_DISABLE;
    channel_cfg.Pulse = 0;
    channel_cfg.OCPolarity = hal_sys::TIM_OCPOLARITY_HIGH;
    channel_cfg.OCMode = hal_sys::TIM_OCMODE_PWM1;

    for ch in [hal_sys::TIM_CHANNEL_3, hal_sys::TIM_CHANNEL_4] {
        assert!(
            hal_sys::HAL_TIM_PWM_ConfigChannel(&raw mut handle, &raw const channel_cfg, ch)
                == hal_sys::HAL_StatusTypeDef_HAL_OK
        );
    }

    hal_msp::timer_post_init(&handle);
    handle
}
