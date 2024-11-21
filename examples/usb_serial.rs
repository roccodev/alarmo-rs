//! Showcases USB-HS using the Alarmo as a USB serial device, without interrupts
//! Mostly adapted from https://github.com/stm32-rs/stm32h7xx-hal/blob/master/examples/usb_serial.rs

#![no_main]
#![no_std]

use alarmo::Alarmo;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::usb_hs::UsbBus;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

// Panic handler is required
use panic_halt as _;

#[entry]
fn main() -> ! {
    let alarmo = unsafe { Alarmo::init() };

    // IMPORTANT! For Alarmo, you need to mask this interrupt, otherwise you will have to handle
    // and clear it. (see the `usb_serial_interrupt` example)
    NVIC::mask(interrupt::OTG_HS);

    static mut USB_BUS_MEM: [u32; 1024] = [0u32; 1024];

    // TODO ref to mutable static
    let usb_bus = unsafe { UsbBus::new(alarmo.usb1, &mut USB_BUS_MEM) };
    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2111, 0x2024))
        .strings(&[usb_device::device::StringDescriptors::default()
            .manufacturer("alarmo-rs")
            .product("usb_serial example")
            .serial_number(env!("CARGO_PKG_VERSION"))])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        // .max_packet_size_0(64).unwrap() <- might benefit your use case
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
        let mut buf = [0u8; 8];
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }
                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
