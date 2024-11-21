#![doc =  include_str!("../README.md")]
#![no_std]

use crate::input::{Buttons, ExtInterrupts};
use core::cell::RefCell;
use dial::Dial;
use stm32h7xx_hal::{
    delay::Delay,
    pac::Peripherals as Stm32Peripherals,
    rcc::{CoreClocks, ResetEnable},
};

#[cfg(all(feature = "alloc", feature = "panic"))]
extern crate alloc; // For panic formatting

pub mod dial;
mod hal_sys;
pub mod input;
mod pac;

#[cfg(feature = "display")]
pub mod display;

#[cfg(feature = "alloc")]
mod e_alloc;

#[cfg(feature = "panic")]
pub mod panic;

static mut DELAY: Option<RefCell<Delay>> = None;

/// Singleton that allows access to the Alarmo's peripherals.
///
/// Downstream binaries must get an instance of this struct by running
/// ```
/// # use alarmo::Alarmo;
///
/// // Only safe if it's the first function called by main
/// let alarmo = unsafe { Alarmo::init() }; // or .init_with_options(...)
/// ```
/// as the first instruction in the `main` function.
///
/// Peripherals are accessed through public fields, allowing for more flexible lifetime constraints.
pub struct Alarmo {
    pub clocks: CoreClocks,
    pub delay: &'static RefCell<Delay>,
    pub dial: Dial,
    pub ext_interrupts: ExtInterrupts,
    pub buttons: Buttons,
    #[cfg(feature = "display")]
    pub display: display::AlarmoDisplay,
    #[cfg(feature = "usb")]
    pub usb1: stm32h7xx_hal::usb_hs::USB1,
}

pub struct AlarmoOptions {
    #[cfg(feature = "alloc")]
    /// The size of the heap in bytes, determines the start address.
    /// The heap is placed in external RAM (OCTOSPI2), so the maximum theoretical size is 32 MiB.
    ///
    /// However, the program text is also stored in that memory region. The current default is a
    /// safe estimate for most programs, but until a better solution is found, the functions
    /// accepting options other than the default will be marked unsafe.
    ///
    /// The default size is 16 MiB.
    pub heap_size: usize,
}

impl Alarmo {
    /// Initializes the Alarmo abstraction layer.
    ///
    /// ## Panics
    /// Panics on future invocations after the first.
    ///
    /// ## Safety
    /// Behavior is undefined if peripherals are accessed/configured before calling this function.
    /// As such, it is recommended to call this function as early as possible, preferably as the
    /// first instruction in `main`.
    pub unsafe fn init() -> Alarmo {
        Self::init_with_options(AlarmoOptions::default())
    }

    pub unsafe fn init_with_options(options: AlarmoOptions) -> Alarmo {
        let mut cortex = cortex_m::Peripherals::take().unwrap();

        cortex.SCB.enable_icache();
        cortex.SCB.enable_dcache(&mut cortex.CPUID);
        cortex_m::interrupt::enable();

        let peripherals = Stm32Peripherals::take().unwrap();
        let pwr = peripherals.PWR.constrain();
        let pwr_cfg = pwr.freeze();
        let rcc = peripherals.RCC.constrain();
        use stm32h7xx_hal::prelude::*;
        let mut ccdr = rcc.freeze(pwr_cfg, &peripherals.SYSCFG);

        // 48MHz clock for USB1
        if cfg!(feature = "usb") {
            let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
            ccdr.peripheral
                .kernel_usb_clk_mux(stm32h7xx_hal::rcc::rec::UsbClkSel::Hsi48);
        }

        #[cfg(feature = "alloc")]
        e_alloc::init_heap(options.heap_size);

        // Split GPIO
        let gpioa = peripherals.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = peripherals.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = peripherals.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiog = peripherals.GPIOG.split_without_reset(ccdr.peripheral.GPIOG);

        #[cfg(feature = "usb")]
        let usb1 = pac::usb::split_usb(
            gpioa.pa11,
            gpioa.pa12,
            peripherals.OTG1_HS_GLOBAL,
            peripherals.OTG1_HS_DEVICE,
            peripherals.OTG1_HS_PWRCLK,
            ccdr.peripheral.USB1OTG,
            &ccdr.clocks,
        );

        // Split timers
        let (dial_timers, disp_timer) = pac::timers::split_timers(
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

        // Split buttons
        let exti = ExtInterrupts {
            syscfg: peripherals.SYSCFG,
            exti: peripherals.EXTI,
            nvic: cortex.NVIC,
        };
        let buttons = Buttons::split(gpiog.pg5, gpiog.pg6, gpioc.pc5);

        // Enable GPIO for FMC and init SRAM
        let disp_pin = unsafe { pac::sram::init(peripherals.FMC, gpioc.pc7) };
        // Enable the FMC clocks for SRAM
        ccdr.peripheral
            .FMC
            .kernel_clk_mux(stm32h7xx_hal::pac::rcc::d1ccipr::FMCSEL_A::Per)
            .enable();

        DELAY = Some(RefCell::new(Delay::new(cortex.SYST, ccdr.clocks)));

        Alarmo {
            delay: DELAY.as_ref().unwrap(),
            clocks: ccdr.clocks,
            dial: Dial {
                timers: dial_timers,
            },
            ext_interrupts: exti,
            buttons,
            #[cfg(feature = "display")]
            display: display::AlarmoDisplay::new(
                disp_timer,
                disp_pin,
                gpiog.pg4,
                DELAY.as_ref().unwrap(),
            ),
            #[cfg(feature = "usb")]
            usb1,
        }
    }
}

impl Default for AlarmoOptions {
    fn default() -> Self {
        AlarmoOptions {
            #[cfg(feature = "alloc")]
            heap_size: 0x1000000, // 16 MiB
        }
    }
}
