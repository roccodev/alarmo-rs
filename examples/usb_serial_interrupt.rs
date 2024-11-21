//! Showcases USB-HS using the Alarmo as a USB serial device, with interrupts
//! Mostly adapted from https://github.com/stm32-rs/stm32h7xx-hal/blob/master/examples/usb_phy_serial_interrupt.rs

#![no_main]
#![no_std]

use alarmo::Alarmo;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::usb_hs::{UsbBus, USB1};
use usb_device::bus::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{DefaultBufferStore, SerialPort};

// Panic handler is required
use panic_halt as _;

static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<UsbBus<USB1>>>>> = Mutex::new(RefCell::new(None));
static SERIAL_PORT: Mutex<
    RefCell<Option<SerialPort<UsbBus<USB1>, DefaultBufferStore, DefaultBufferStore>>>,
> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let alarmo = unsafe { Alarmo::init() };

    // These buffers need to live long enough for the device/port that are shared with interrupts.
    static mut USB_BUS_MEM: [u32; 1024] = [0u32; 1024];
    static mut USB_BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus<USB1>>> = None;

    let usb_serial = unsafe {
        // TODO alternative to mutable static ref
        USB_BUS_ALLOCATOR = Some(UsbBus::new(alarmo.usb1, &mut USB_BUS_MEM));
        SerialPort::new(USB_BUS_ALLOCATOR.as_ref().unwrap())
    };
    let usb_dev = UsbDeviceBuilder::new(
        unsafe { USB_BUS_ALLOCATOR.as_ref().unwrap() },
        UsbVidPid(0x2111, 0x2024),
    )
    .strings(&[usb_device::device::StringDescriptors::default()
        .manufacturer("alarmo-rs")
        .product("usb_serial_interrupt example")
        .serial_number(env!("CARGO_PKG_VERSION"))])
    .unwrap()
    .device_class(usbd_serial::USB_CLASS_CDC)
    // .max_packet_size_0(64).unwrap() <- might benefit your use case
    .build();

    cortex_m::interrupt::free(|cs| {
        USB_DEVICE.borrow(cs).replace(Some(usb_dev));
        SERIAL_PORT.borrow(cs).replace(Some(usb_serial));
    });

    loop {}
}

#[interrupt]
fn OTG_HS() {
    // USB logic goes here.
    // From https://github.com/stm32-rs/stm32h7xx-hal/blob/master/examples/usb_phy_serial_interrupt.rs
    cortex_m::interrupt::free(|cs| {
        if let (Some(port), Some(device)) = (
            SERIAL_PORT.borrow(cs).borrow_mut().as_mut(),
            USB_DEVICE.borrow(cs).borrow_mut().as_mut(),
        ) {
            if device.poll(&mut [port]) {
                let mut buf = [0u8; 8];
                // Make sure to read to clear the interrupt!
                match port.read(&mut buf) {
                    Ok(count) if count > 0 => {
                        // Echo back in upper case
                        for c in buf[0..count].iter_mut() {
                            if 0x61 <= *c && *c <= 0x7a {
                                *c &= !0x20;
                            }
                        }

                        let mut write_offset = 0;
                        while write_offset < count {
                            match port.write(&buf[write_offset..count]) {
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
    })
}
