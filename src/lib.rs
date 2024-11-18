#![no_std]

use core::arch::global_asm;

use dial::Dial;
use display::SelectPin;
use pac::timers::Timers;
use stm32h7xx_hal::{
    delay::Delay,
    gpio::{GpioExt, Pin},
    pac::Peripherals as Stm32Peripherals,
    pwr::PwrExt,
    rcc::{CoreClocks, RccExt, ResetEnable},
};

mod arch;
pub mod dial;
mod hal_sys;
mod interrupt_handlers;
mod pac;

// Include startup code
#[cfg(target_arch = "arm")]
global_asm!(core::include_str!("../vendor/startup_stm32h730xx.s"));

#[cfg(feature = "display")]
pub mod display;

static mut ALARMO: Option<Alarmo> = None;

pub struct Alarmo {
    pub clocks: CoreClocks,
    pub delay: Delay,

    timers: Timers,
    display_pins: Option<(Pin<'G', 4>, SelectPin)>,
}

impl Alarmo {
    pub unsafe fn init() -> &'static mut Alarmo {
        let mut cortex = cortex_m::Peripherals::take().unwrap();

        arch::enable_instruction_cache(&mut cortex);
        arch::enable_data_cache(&mut cortex);
        arch::enable_interrupts();

        let peripherals = Stm32Peripherals::take().unwrap();
        let pwr = peripherals.PWR.constrain();
        let pwr_cfg = pwr.freeze();
        let rcc = peripherals.RCC.constrain();
        let ccdr = rcc.freeze(pwr_cfg, &peripherals.SYSCFG);

        // Split GPIO
        let gpioa = peripherals.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = peripherals.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = peripherals.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiog = peripherals.GPIOG.split_without_reset(ccdr.peripheral.GPIOG);

        // Split timers
        let timers = Timers::new(
            &ccdr.clocks,
            peripherals.TIM1,
            peripherals.TIM3,
            gpioa.pa8,
            gpioa.pa10,
            gpiob.pb1,
            gpioc.pc8,
            ccdr.peripheral.TIM1,
            ccdr.peripheral.TIM3,
        );

        // Enable GPIO for FMC and init SRAM
        let disp_pin = pac::sram::init(peripherals.FMC, gpioc.pc7);
        // Enable the FMC clocks for SRAM
        ccdr.peripheral
            .FMC
            .kernel_clk_mux(stm32h7xx_hal::pac::rcc::d1ccipr::FMCSEL_A::Per)
            .enable();

        ALARMO = Some(Alarmo {
            delay: Delay::new(cortex.SYST, ccdr.clocks),
            clocks: ccdr.clocks,
            timers,
            display_pins: Some((gpiog.pg4, disp_pin)),
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
        let (rs, sel) = self.display_pins.take().expect("display already init");
        display::AlarmoDisplayInterface::init(&mut self.timers, rs, sel)
    }
}
