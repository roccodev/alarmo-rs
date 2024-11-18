#![no_std]

use dial::Dial;
use hal_sys::{FMC_NORSRAM_TimingTypeDef, SRAM_HandleTypeDef};
use pac::timers::Timers;
use stm32h7xx_hal::{
    delay::Delay,
    gpio::GpioExt,
    pac::Peripherals as Stm32Peripherals,
    pwr::PwrExt,
    rcc::{CoreClocks, RccExt, ResetEnable},
};

mod arch;
pub mod dial;
mod hal_msp;
#[allow(warnings)]
pub mod hal_sys;
mod interrupt_handlers;
mod pac;

#[cfg(feature = "display")]
pub mod display;

static mut ALARMO: Option<Alarmo> = None;

pub struct Alarmo {
    pub clocks: CoreClocks,
    pub delay: Delay,

    sram_handle: SRAM_HandleTypeDef,
    timers: Timers,
}

impl Alarmo {
    pub unsafe fn init() -> &'static mut Alarmo {
        let mut cortex = cortex_m::Peripherals::take().unwrap();

        arch::enable_instruction_cache(&mut cortex);
        arch::enable_data_cache(&mut cortex);
        arch::enable_interrupts();
        hal_sys::HAL_Init();

        let peripherals = Stm32Peripherals::take().unwrap();
        let pwr = peripherals.PWR.constrain();
        let pwr_cfg = pwr.freeze();
        let rcc = peripherals.RCC.constrain();
        let ccdr = rcc.freeze(pwr_cfg, &peripherals.SYSCFG);

        // Split timers
        let timers = Timers::new(
            &ccdr.clocks,
            peripherals.TIM1,
            peripherals.TIM3,
            peripherals.GPIOA.split(ccdr.peripheral.GPIOA),
            peripherals.GPIOB.split(ccdr.peripheral.GPIOB),
            peripherals.GPIOC.split(ccdr.peripheral.GPIOC),
            ccdr.peripheral.TIM1,
            ccdr.peripheral.TIM3,
        );

        let sram_handle = init_sram();
        // Enable the FMC clocks for SRAM
        ccdr.peripheral
            .FMC
            .kernel_clk_mux(stm32h7xx_hal::pac::rcc::d1ccipr::FMCSEL_A::Per)
            .enable();

        ALARMO = Some(Alarmo {
            delay: Delay::new(cortex.SYST, ccdr.clocks),
            clocks: ccdr.clocks,
            sram_handle,
            timers,
        });
        ALARMO.as_mut().unwrap()
    }

    pub fn dial(&mut self) -> Dial {
        Dial {
            timers: &mut self.timers,
        }
    }

    #[cfg(feature = "display")]
    pub unsafe fn init_display(&mut self) -> display::AlarmoDisplayInterface {
        display::AlarmoDisplayInterface::init(&mut self.timers)
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
