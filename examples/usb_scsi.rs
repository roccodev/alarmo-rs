//! Announces the Alarmo as a USB mass storage device, forwarding read-only access to the eMMC.
//! Uses https://github.com/apohrebniak/usbd-storage/blob/master/examples/src/bin/stm32f411x_scsi_bbb.rs

#![no_main]
#![no_std]

extern crate alloc;

use cortex_m::prelude::*;
use stm32h7xx_hal::prelude::*;

use alarmo::{Alarmo, AlarmoOptions};
use alloc::vec;
use alloc::vec::Vec;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::pac::SDMMC2;
use stm32h7xx_hal::sdmmc::{Emmc, Sdmmc};
use stm32h7xx_hal::usb_hs::{UsbBus, USB1};
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid};

use usbd_storage::subclass::scsi::{Scsi, ScsiCommand};
use usbd_storage::subclass::Command;
use usbd_storage::transport::bbb::{BulkOnly, BulkOnlyError};
use usbd_storage::transport::TransportError;

#[derive(Default)]
struct State {
    storage_offset: usize,
    sense_key: Option<u8>,
    sense_key_code: Option<u8>,
    sense_qualifier: Option<u8>,

    // Caching the read buffer for the current request being handled is almost mandatory, the
    // eMMC is quite slow
    last_lba_len: (u32, u32),
    cur_buf: Option<Vec<u8>>,
}

impl State {
    fn reset(&mut self) {
        self.storage_offset = 0;
        self.last_lba_len = (0, 0);
        self.sense_key = None;
        self.sense_key_code = None;
        self.sense_qualifier = None;
        self.cur_buf = None;
    }
}

#[entry]
fn main() -> ! {
    let alarmo = unsafe {
        Alarmo::init_with_options(AlarmoOptions {
            // Higher clock necessary to avoid USB timeouts
            sys_ck: Some(200.MHz()),
            ..Default::default()
        })
    };

    // Init emmc, frequency must be 26 MHz or less during init, can be changed after init with
    // `emmc.set_bus`
    let mut emmc = alarmo.emmc;
    while emmc.init(26.MHz()).is_err() {
        alarmo.delay.borrow_mut().delay_ms(1000u32);
    }
    let card = emmc.card().unwrap();
    let usb_block_count = card.ext_csd.sector_count().swap_bytes();
    let usb_block_size = 512;

    // IMPORTANT! For Alarmo, you need to mask this interrupt, otherwise you will have to handle
    // and clear it.
    NVIC::mask(interrupt::OTG_HS);

    static mut USB_BUS_MEM: [u32; 1024] = [0u32; 1024];
    static mut TRANSPORT_BUF: [u8; 512] = [0u8; 512];

    let usb_bus = UsbBus::new(alarmo.usb1, unsafe { USB_BUS_MEM.as_mut_slice() });
    let mut scsi = Scsi::new(&usb_bus, 64, 0, unsafe { TRANSPORT_BUF.as_mut_slice() }).unwrap();
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2111, 0x2024))
        .strings(&[usb_device::device::StringDescriptors::default()
            .manufacturer("alarmo-rs")
            .product("usb_scsi example")
            .serial_number(env!("CARGO_PKG_VERSION"))])
        .unwrap()
        .self_powered(false)
        .max_packet_size_0(64)
        .unwrap()
        .build();

    let mut state = State::default();

    loop {
        if !usb_dev.poll(&mut [&mut scsi]) {
            continue;
        }
        if matches!(usb_dev.state(), UsbDeviceState::Default) {
            state.reset();
        }

        let _ = scsi.poll(|command| {
            scsi_process(
                command,
                &mut emmc,
                &mut state,
                usb_block_size as u32,
                usb_block_count,
            )
            .unwrap();
        });
    }
}

fn scsi_process(
    mut command: Command<ScsiCommand, Scsi<BulkOnly<UsbBus<USB1>, &mut [u8]>>>,
    emmc: &mut Sdmmc<SDMMC2, Emmc>,
    state: &mut State,
    block_size: u32,
    block_count: u32,
) -> Result<(), TransportError<BulkOnlyError>> {
    // Most of this is boilerplate to make SCSI work. Actual reading is in the ::Read handler
    match command.kind {
        ScsiCommand::TestUnitReady { .. } => {
            command.pass();
        }
        ScsiCommand::Inquiry { .. } => {
            command.try_write_data_all(&[
                0x00, // periph qualifier, periph device type
                0x80, // Removable
                0x04, // SPC-2 compliance
                0x02, // NormACA, HiSu, Response data format
                0x20, // 36 bytes in total
                0x00, // additional fields, none set
                0x00, // additional fields, none set
                0x00, // additional fields, none set
                b'a', b'l', b'a', b'r', b'm', b'o', b'r', b's', // 8-byte T-10 vendor id
                b'u', b's', b'b', b'_', b's', b'c', b's', b'i', b'_', b'e', b'x', b'a', b'm', b'p',
                b'l', b'e', // 16-byte product identification
                b'1', b'.', b'0', b'0', // 4-byte product revision
            ])?;
            command.pass();
        }
        ScsiCommand::RequestSense { .. } => {
            command.try_write_data_all(&[
                0x70,                         // RESPONSE CODE. Set to 70h for information on current errors
                0x00,                         // obsolete
                state.sense_key.unwrap_or(0), // Bits 3..0: SENSE KEY. Contains information describing the error.
                0x00,
                0x00,
                0x00,
                0x00, // INFORMATION. Device-specific or command-specific information.
                0x00, // ADDITIONAL SENSE LENGTH.
                0x00,
                0x00,
                0x00,
                0x00,                               // COMMAND-SPECIFIC INFORMATION
                state.sense_key_code.unwrap_or(0),  // ASC
                state.sense_qualifier.unwrap_or(0), // ASCQ
                0x00,
                0x00,
                0x00,
                0x00,
            ])?;
            state.reset();
            command.pass();
        }
        ScsiCommand::ReadCapacity10 { .. } => {
            let mut data = [0u8; 8];
            let _ = &mut data[0..4].copy_from_slice(&u32::to_be_bytes(block_count - 1));
            let _ = &mut data[4..8].copy_from_slice(&u32::to_be_bytes(block_size));
            command.try_write_data_all(&data)?;
            command.pass();
        }
        ScsiCommand::ReadCapacity16 { .. } => {
            let mut data = [0u8; 16];
            let _ = &mut data[0..8].copy_from_slice(&u32::to_be_bytes(block_count - 1));
            let _ = &mut data[8..12].copy_from_slice(&u32::to_be_bytes(block_size));
            command.try_write_data_all(&data)?;
            command.pass();
        }
        ScsiCommand::ReadFormatCapacities { .. } => {
            let mut data = [0u8; 12];
            let _ = &mut data[0..4].copy_from_slice(&[
                0x00, 0x00, 0x00, 0x08, // capacity list length
            ]);
            let _ = &mut data[4..8].copy_from_slice(&u32::to_be_bytes(block_count)); // number of blocks
            data[8] = 0x01; //unformatted media
            let block_length_be = u32::to_be_bytes(block_size);
            data[9] = block_length_be[1];
            data[10] = block_length_be[2];
            data[11] = block_length_be[3];

            command.try_write_data_all(&data)?;
            command.pass();
        }
        ScsiCommand::Read { lba, len } => {
            let lba = lba as u32;
            let len = len as u32;

            if state.last_lba_len != (lba, len) {
                // Request changed, discard buffer
                state.storage_offset = 0;
                state.cur_buf = None;
            }
            state.last_lba_len = (lba, len);

            if state.storage_offset != (len * block_size) as usize {
                // Partial read or no read was started, finish it

                let mut buf = if state.cur_buf.is_none() {
                    let mut buf = vec![0u8; block_size as usize * len as usize];
                    // Read blocks from eMMC
                    if let Err(e) = emmc.read_blocks(lba, &mut buf) {
                        panic!("Error reading blocks {lba} x{len}: {:?}", e);
                    }
                    buf
                } else {
                    state.cur_buf.take().unwrap()
                };

                // Send data via USB
                let count = command.write_data(&mut buf[state.storage_offset..])?;
                state.cur_buf = Some(buf);
                state.storage_offset += count;
            } else {
                // Read finished, signal result
                command.pass();
                state.storage_offset = 0;
            }
        }
        ScsiCommand::Write { .. } => {
            // Writing is not supported
            state.sense_key.replace(0x07); // "Data protect"
            state.sense_key_code.replace(0x27); // Invalid command operation ASC
            state.sense_qualifier.replace(0x00); // Invalid command operation ASCQ
            command.fail();
        }
        ScsiCommand::ModeSense6 { .. } => {
            command.try_write_data_all(&[
                0x03, // number of bytes that follow
                0x00, // the media type is SBC
                0x80, // write-protected, no cache-control bytes support
                0x00, // no mode-parameter block descriptors
            ])?;
            command.pass();
        }
        ScsiCommand::ModeSense10 { .. } => {
            command.try_write_data_all(&[0x00, 0x06, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00])?;
            command.pass();
        }
        _ => {
            state.sense_key.replace(0x05); // illegal request Sense Key
            state.sense_key_code.replace(0x20); // Invalid command operation ASC
            state.sense_qualifier.replace(0x00); // Invalid command operation ASCQ
            command.fail();
        }
    }

    Ok(())
}
