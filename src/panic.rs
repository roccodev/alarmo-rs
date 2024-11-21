//! Provides a panic handler that displays the panic message on the Alarmo's display

use crate::display::HalDelay;
use crate::{display, pac};
use core::cell::RefCell;
use core::panic::PanicInfo;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Dimensions, DrawTarget, RgbColor};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation};
use stm32h7xx_hal::delay::Delay;
use stm32h7xx_hal::gpio::GpioExt;
use stm32h7xx_hal::prelude::{_stm32h7xx_hal_pwr_PwrExt, _stm32h7xx_hal_rcc_RccExt};
use stm32h7xx_hal::rcc::ResetEnable;
use stm32h7xx_hal::stm32::Peripherals;

// No inline to allow for debugging
#[inline(never)]
#[panic_handler]
unsafe fn panic(info: &PanicInfo) -> ! {
    handle_panic(info)
}

struct PanicData {
    message: [u8; 256],
    message_len: usize,
    file: [u8; 64],
    file_len: usize,
    line: u32,
}

unsafe fn handle_panic(info: &PanicInfo) -> ! {
    #[cfg(not(feature = "alloc"))]
    let message = info.message().as_str();
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;
    #[cfg(feature = "alloc")]
    let message = Some(info.message().to_string());

    let location = info.location();
    let mut panic_data = PanicData {
        message: [0u8; 256],
        message_len: 0,
        file: [0u8; 64],
        file_len: 0,
        line: 10,
    };
    if let Some(message) = message {
        let take = message.len().min(panic_data.message.len());
        panic_data.message[..take].copy_from_slice(&message.as_bytes()[..take]);
        panic_data.message_len = take;
    }
    if let Some(location) = location {
        let take = location.file().len().min(panic_data.file.len());
        panic_data.line = location.line();
        panic_data.file[..take].copy_from_slice(&location.file().as_bytes()[..take]);
        panic_data.file_len = take;
    } else {
        panic_data.line = u32::MAX;
    }

    show_panic(panic_data);

    loop {}
}

unsafe fn show_panic(data: PanicData) {
    // Take full control and prepare a barebones view of the panic message
    cortex_m::interrupt::disable();

    let cortex = cortex_m::Peripherals::steal();
    let peripherals = Peripherals::steal();

    let pwr = peripherals.PWR.constrain();
    let pwr_cfg = pwr.freeze();
    let rcc = peripherals.RCC.constrain();
    let ccdr = rcc.freeze(pwr_cfg, &peripherals.SYSCFG);

    // Split GPIO - only the ones needed for display
    let gpiob = peripherals.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioc = peripherals.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiog = peripherals.GPIOG.split_without_reset(ccdr.peripheral.GPIOG);

    // Split timers
    let (disp_timer, _) = pac::timers::display_timer(
        &ccdr.clocks,
        peripherals.TIM3,
        gpiob.pb1,
        gpioc.pc8,
        ccdr.peripheral.TIM3,
    );

    // Init FMC clocks and SRAM
    let disp_pin = unsafe { pac::sram::init(peripherals.FMC, gpioc.pc7) };
    ccdr.peripheral
        .FMC
        .kernel_clk_mux(stm32h7xx_hal::pac::rcc::d1ccipr::FMCSEL_A::Per)
        .enable();

    // Access is safe because we are the only accessor at this point
    crate::DELAY = Some(RefCell::new(Delay::new(cortex.SYST, ccdr.clocks)));

    let message = core::str::from_utf8(&data.message[..data.message_len])
        .unwrap_or("malformed panic message");
    let file = core::str::from_utf8(&data.file[..data.file_len]).unwrap_or("malformed panic file");
    let mut line = [0u8; u32::MAX.ilog10() as usize + 1];
    let line_len = itoa(data.line, &mut line);
    let line = core::str::from_utf8(&line[..line_len]).unwrap_or("??");

    let mut disp = display::AlarmoDisplay::new(
        disp_timer,
        disp_pin,
        gpiog.pg4,
        crate::DELAY.as_ref().unwrap(),
    );
    disp.hard_reset();
    disp.set_backlight(1.0);

    let mut delay = HalDelay;
    let mut disp = mipidsi::Builder::new(ST7789, disp)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation {
            mirrored: false,
            rotation: mipidsi::options::Rotation::Deg270,
        })
        .init(&mut delay)
        .unwrap();
    disp.clear(Rgb565::BLUE).ok();

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    // Draw centered title
    Text::with_alignment("!! PANIC !!", Point::new(160, 30), style, Alignment::Center)
        .draw(&mut disp)
        .ok();

    // Draw panic message, this possibly spans multiple lines, so get the lowest point to properly
    // position the next items
    let text_msg = Text::new(message, Point::new(15, 50), style);
    let max_y = text_msg
        .bounding_box()
        .bottom_right()
        .map(|p| p.y)
        .unwrap_or_default();
    text_msg.draw(&mut disp).ok();

    // Draw file name and line
    Text::new(file, Point::new(15, max_y + 10), style)
        .draw(&mut disp)
        .ok();
    Text::new(line, Point::new(15, max_y + 30), style)
        .draw(&mut disp)
        .ok();
}

fn itoa(mut num: u32, buf: &mut [u8]) -> usize {
    if num == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut chars = 0;
    while num > 0 {
        buf[chars] = char::from_digit(num % 10, 10).map(|c| c as u8).unwrap();
        num /= 10;
        chars += 1;
    }
    buf[..chars].reverse();
    chars
}
