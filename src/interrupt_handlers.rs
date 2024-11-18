//! Interrupt handlers exported for access by the startup/linker layout script

#[unsafe(no_mangle)]
pub unsafe extern "C" fn SysTick_Handler() {}

#[unsafe(no_mangle)]
pub extern "C" fn NMI_Handler() {}

#[unsafe(no_mangle)]
pub extern "C" fn HardFault_Handler() {
    panic!("HardFault");
}

#[unsafe(no_mangle)]
pub extern "C" fn MemManage_Handler() {
    panic!("MemManage");
}

#[unsafe(no_mangle)]
pub extern "C" fn BusFault_Handler() {
    panic!("BusFault");
}

#[unsafe(no_mangle)]
pub extern "C" fn UsageFault_Handler() {
    panic!("UsageFault");
}

#[unsafe(no_mangle)]
pub extern "C" fn SVC_Handler() {}

#[unsafe(no_mangle)]
pub extern "C" fn DebugMon_Handler() {}

#[unsafe(no_mangle)]
pub extern "C" fn PendSV_Handler() {}
