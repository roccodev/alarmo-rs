use stm32h7xx_hal::gpio::{PA11, PA12};
use stm32h7xx_hal::pac::{OTG1_HS_DEVICE, OTG1_HS_PWRCLK};
use stm32h7xx_hal::rcc::rec::Usb1Otg;
use stm32h7xx_hal::rcc::CoreClocks;
use stm32h7xx_hal::stm32::OTG1_HS_GLOBAL;
use stm32h7xx_hal::usb_hs::USB1;

pub fn split_usb(
    pin_dm: PA11,
    pin_dp: PA12,
    otg1_hs_global: OTG1_HS_GLOBAL,
    otg1_hs_device: OTG1_HS_DEVICE,
    otg1_hs_pwrclk: OTG1_HS_PWRCLK,
    peripheral: Usb1Otg,
    clocks: &CoreClocks,
) -> USB1 {
    USB1::new(
        otg1_hs_global,
        otg1_hs_device,
        otg1_hs_pwrclk,
        pin_dm,
        pin_dp,
        peripheral,
        clocks,
    )
}
