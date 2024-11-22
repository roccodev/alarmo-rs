use crate::delay::HalDelay;
use core::ptr::addr_of;
use stm32h7xx_hal::adc::{self, AdcDmaMode, AdcLshift, AdcSampleTime, Resolution};
use stm32h7xx_hal::dma::config::Priority;
use stm32h7xx_hal::dma::dma::{DmaConfig, StreamsTuple};
use stm32h7xx_hal::dma::Transfer;
use stm32h7xx_hal::gpio::{PB0, PC4};
use stm32h7xx_hal::pac::{ADC1, ADC2, DMA1};
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::rcc::rec::{Adc12, Dma1};
use stm32h7xx_hal::rcc::CoreClocks;

pub struct AdcBuffers {
    pub adc1: *const u16,
    pub adc2: *const u16,
}

#[repr(C, align(32))]
struct AdcBufRepr {
    adc1: [u16; 3],
    _pad: [u8; 26],
    adc2: [u16; 2],
}

static mut ADC_BUFFERS: AdcBufRepr = AdcBufRepr {
    adc1: [0u16; 3],
    _pad: [0u8; 26],
    adc2: [0u16; 2],
};

pub fn split_adc(
    adc1: ADC1,
    adc2: ADC2,
    pc4: PC4,
    pb0: PB0,
    dma1: DMA1,
    rcc_adc: Adc12,
    rcc_dma1: Dma1,
    clocks: &CoreClocks,
) -> AdcBuffers {
    unsafe {
        // Aligned to a cache line (which also implies word and half-word alignment for atomic access)
        assert_eq!(addr_of!(ADC_BUFFERS.adc1) as usize % 32, 0);
        assert_eq!(addr_of!(ADC_BUFFERS.adc2) as usize % 32, 0);
    }

    let (mut adc1, mut adc2) = adc::adc12(adc1, adc2, 4.MHz(), &mut HalDelay, rcc_adc, clocks);
    adc1.calibrate();
    adc2.calibrate();

    let (mut adc1, mut adc2) = (adc1.enable(), adc2.enable());

    adc1.set_resolution(Resolution::SixteenBit);
    adc1.set_lshift(AdcLshift::default());
    adc1.set_sample_time(AdcSampleTime::T_64);

    adc2.set_resolution(Resolution::SixteenBit);
    adc2.set_lshift(AdcLshift::default());
    adc2.set_sample_time(AdcSampleTime::T_64);

    let mut ch1_2 = pb0.into_analog();
    let mut ch1_4 = pc4.into_analog();
    let dma_cfg = DmaConfig::default()
        .circular_buffer(true)
        .priority(Priority::Low)
        .fifo_enable(false)
        .peripheral_increment(false)
        .memory_increment(false);

    let streams = StreamsTuple::new(dma1, rcc_dma1);
    let mut tx1 = Transfer::init(
        streams.1,
        adc1,
        unsafe { ADC_BUFFERS.adc1.as_mut_slice() },
        None,
        dma_cfg,
    );
    let mut tx2 = Transfer::init(
        streams.2,
        adc2,
        unsafe { ADC_BUFFERS.adc2.as_mut_slice() },
        None,
        dma_cfg,
    );

    tx1.start(|adc| {
        adc.start_conversion_dma(&mut ch1_4, AdcDmaMode::Circular);
    });
    tx2.start(|adc| {
        adc.start_conversion_dma(&mut ch1_2, AdcDmaMode::Circular);
    });

    // Don't stop transfers when function returns
    core::mem::forget(tx1);
    core::mem::forget(tx2);

    unsafe {
        // Keep buffers as pointers to convey that accessing the data is unsafe
        AdcBuffers {
            adc1: ADC_BUFFERS.adc1.as_ptr(),
            adc2: ADC_BUFFERS.adc2.as_ptr(),
        }
    }
}
