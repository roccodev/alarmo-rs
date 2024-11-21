use stm32h7xx_hal::gpio::{Speed, PB4, PD6, PD7, PG10, PG11, PG9};
use stm32h7xx_hal::pac::SDMMC2;
use stm32h7xx_hal::rcc::rec::Sdmmc2;
use stm32h7xx_hal::rcc::CoreClocks;
use stm32h7xx_hal::sdmmc::{Buswidth, Emmc, Sdmmc, SdmmcExt};

pub fn split_emmc(
    pd6: PD6,
    pd7: PD7,
    pg9: PG9,
    pg10: PG10,
    pg11: PG11,
    pb4: PB4,
    peripheral: SDMMC2,
    rcc: Sdmmc2,
    clocks: &CoreClocks,
) -> Sdmmc<SDMMC2, Emmc> {
    let _ = (
        pd6.into_alternate::<11>().speed(Speed::VeryHigh),
        pd7.into_alternate::<11>().speed(Speed::VeryHigh),
        pg9.into_alternate::<11>().speed(Speed::VeryHigh),
        pg10.into_alternate::<11>().speed(Speed::VeryHigh),
        pg11.into_alternate::<10>().speed(Speed::VeryHigh),
        pb4.into_alternate::<9>().speed(Speed::VeryHigh),
        // Other pins for 8-bus SDMMC. I tried enabling them, but the data gets corrupted
        // (some bits are flipped)
        // pb8.into_alternate::<9>().speed(Speed::VeryHigh),
        // pb9.into_alternate::<9>().speed(Speed::VeryHigh),
        // pg13.into_alternate::<10>().speed(Speed::VeryHigh),
        // pg14.into_alternate::<10>().speed(Speed::VeryHigh),
    );
    // Official firmware also does this, doesn't seem to be needed though
    // pe1.into_push_pull_output().set_high();
    peripheral.sdmmc_unchecked(Buswidth::Four, rcc, &clocks)
}
