#![no_std]
#![no_main]

use alarmo::Alarmo;
use panic_halt as _;

static mut TEST: Option<Alarmo> = None;

#[no_mangle]
pub unsafe fn main() {
    TEST = Some(Alarmo::init());
    alarmo::GaryMain(&raw const TEST.as_ref().unwrap().tim3_handle);
}
