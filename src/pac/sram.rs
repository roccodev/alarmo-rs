use crate::hal_sys;
use stm32h7xx_hal::{
    gpio::{Output, Pin, PushPull},
    pac::{self, FMC},
};

pub unsafe fn init(fmc: FMC, pc7: Pin<'C', 7>) -> Pin<'C', 7, Output<PushPull>> {
    init_gpio('A', &[4], 12);
    init_gpio('F', &[12], 12);
    init_gpio('B', &[14, 15], 12);
    init_gpio('D', &[0, 1, 4, 5, 8, 9, 10, 14, 15], 12);
    init_gpio('E', &[7, 8, 9, 10, 12, 15], 12);

    let mut pc7 = pc7.into_alternate::<9>().into_push_pull_output();
    pc7.set_internal_resistor(stm32h7xx_hal::gpio::Pull::None);
    pc7.set_high();

    fmc_norsram_init(&fmc);
    fmc_norsram_timing_init(&fmc);
    fmc_norsram_extended_timing_init(&fmc);

    // Enable FMC
    fmc.bcr1
        .modify(|_, w| w.mbken().set_bit().fmcen().set_bit());

    let swap_cfg = hal_sys::FMC_SWAPBMAP_SDRAM_SRAM;
    let mask = 0x3 << hal_sys::FMC_BCR1_BMAP_Pos;
    fmc.bcr1
        .modify(|old, w| w.bits(((old.bits()) & (!(mask))) | (swap_cfg)));

    pc7
}

unsafe fn fmc_norsram_init(fmc: &FMC) {
    fmc.bcr1.modify(|_, w| w.mbken().clear_bit());

    // FMC_NORSRAM_Init start
    let flash_access = hal_sys::FMC_NORSRAM_FLASH_ACCESS_ENABLE;

    // zeroed fields:
    // ExtendedMode, AsynchronousWait, WriteBurst, ContinuousClock, WriteFifo, PageSize
    let btcr = flash_access
        | hal_sys::FMC_DATA_ADDRESS_MUX_DISABLE
        | hal_sys::FMC_MEMORY_TYPE_SRAM
        | hal_sys::FMC_NORSRAM_MEM_BUS_WIDTH_16
        | hal_sys::FMC_BURST_ACCESS_MODE_DISABLE
        | hal_sys::FMC_WAIT_SIGNAL_POLARITY_LOW
        | 0 // WaitSignalActive
        | hal_sys::FMC_WRITE_OPERATION_ENABLE;

    let mask = hal_sys::FMC_BCRx_MBKEN
        | hal_sys::FMC_BCRx_MUXEN
        | hal_sys::FMC_BCRx_MTYP
        | hal_sys::FMC_BCRx_MWID
        | hal_sys::FMC_BCRx_FACCEN
        | hal_sys::FMC_BCRx_BURSTEN
        | hal_sys::FMC_BCRx_WAITPOL
        | hal_sys::FMC_BCRx_WAITCFG
        | hal_sys::FMC_BCRx_WREN
        | hal_sys::FMC_BCRx_WAITEN
        | hal_sys::FMC_BCRx_EXTMOD
        | hal_sys::FMC_BCRx_ASYNCWAIT
        | hal_sys::FMC_BCRx_CBURSTRW
        | hal_sys::FMC_BCR1_CCLKEN
        | hal_sys::FMC_BCR1_WFDIS
        | hal_sys::FMC_BCRx_CPSIZE;

    fmc.bcr1
        .modify(|old, w| w.bits(((old.bits()) & (!(mask))) | (btcr)));
}

unsafe fn fmc_norsram_timing_init(fmc: &FMC) {
    let timings = (2 << hal_sys::FMC_BTRx_ADDSET_Pos)
        | (0 << hal_sys::FMC_BTRx_ADDHLD_Pos)
        | (2 << hal_sys::FMC_BTRx_DATAST_Pos)
        | (0 << hal_sys::FMC_BTRx_BUSTURN_Pos)
        | (1u32.wrapping_sub(1) << hal_sys::FMC_BTRx_CLKDIV_Pos)
        | (2u32.wrapping_sub(2) << hal_sys::FMC_BTRx_DATLAT_Pos)
        | hal_sys::FMC_ACCESS_MODE_A;
    fmc.btr1.write(|w| w.bits(timings));
}

unsafe fn fmc_norsram_extended_timing_init(fmc: &FMC) {
    // TODO: ??
    let extended_mode = false;
    let mut mask = 0;

    let timings = if extended_mode {
        mask = hal_sys::FMC_BWTRx_ADDSET
            | hal_sys::FMC_BWTRx_ADDHLD
            | hal_sys::FMC_BWTRx_DATAST
            | hal_sys::FMC_BWTRx_BUSTURN
            | hal_sys::FMC_BWTRx_ACCMOD;
        // ---------
        (2 << hal_sys::FMC_BWTRx_ADDSET_Pos)
            | (0 << hal_sys::FMC_BWTRx_ADDHLD_Pos)
            | (2 << hal_sys::FMC_BWTRx_DATAST_Pos)
            | (0 << hal_sys::FMC_BWTRx_BUSTURN_Pos)
            | hal_sys::FMC_ACCESS_MODE_A
    } else {
        0x0FFFFFFF
    };
    fmc.bwtr1
        .modify(|old, w| w.bits(((old.bits()) & (!(mask))) | (timings)));
}

/// Adaptation of the various `Pin` functions to make it more practical to enable pins in bulk
unsafe fn init_gpio(port: char, pins: &[u32], af: u32) {
    let ptr = match port {
        'A' => pac::GPIOA::ptr(),
        'B' => pac::GPIOB::ptr() as _,
        'C' => pac::GPIOC::ptr() as _,
        'D' => pac::GPIOD::ptr() as _,
        'E' => pac::GPIOE::ptr() as _,
        'F' => pac::GPIOF::ptr() as _,
        'G' => pac::GPIOG::ptr() as _,
        'H' => pac::GPIOH::ptr() as _,
        'J' => pac::GPIOJ::ptr() as _,
        'K' => pac::GPIOK::ptr() as _,
        _ => panic!("Unknown GPIO port"),
    };
    let mode = crate::hal_sys::MODE_AF;
    for &pin in pins {
        let offset = 2 * pin;
        (*ptr)
            .otyper
            .modify(|r, w| w.bits(r.bits() & !(0b1 << pin) | (0 << pin)));
        if pin < 8 {
            let offset2 = 4 * pin;
            (*ptr)
                .afrl
                .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset2)) | (af << offset2)));
        } else {
            let offset2 = 4 * (pin - 8);
            (*ptr)
                .afrh
                .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset2)) | (af << offset2)));
        }
        (*ptr)
            .moder
            .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (mode << offset)));
        (*ptr).ospeedr.modify(|r, w| {
            w.bits((r.bits() & !(0b11 << offset)) | (hal_sys::GPIO_SPEED_FREQ_MEDIUM << offset))
        });
    }
}
