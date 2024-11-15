use cortex_m::{
    asm::{dsb, isb},
    Peripherals,
};

use crate::hal_sys;

#[inline]
pub unsafe fn enable_instruction_cache(cortex: &mut Peripherals) {
    cortex.SCB.enable_icache();
}

#[inline]
pub unsafe fn enable_data_cache(cortex: &mut Peripherals) {
    cortex.SCB.enable_dcache(&mut cortex.CPUID);
}

#[inline]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("cpsie i");
}
