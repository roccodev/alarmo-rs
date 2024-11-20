use core::cell::RefCell;
use cortex_m::interrupt::{CriticalSection, Mutex};
use cortex_m::peripheral::NVIC;
use stm32h7xx_hal::gpio::{Edge, ExtiPin, Input, Pin};
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::pac::{EXTI, SYSCFG};

pub struct Buttons {
    pin_mail: Pin<'G', 5, Input>,
    pin_back: Pin<'G', 6, Input>,
    pin_dial: Pin<'C', 5, Input>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Button {
    Mail,
    Back,
    DialClick,
}

/// Defines which button to enable when using interrupt-based input.
///
/// With interrupt-based input, it is not possible to enable both the mail and dial click buttons.
/// Interrupts are registered via EXTI, which only allows one GPIO port per line. Because the
/// mail and dial click buttons share the same line (no. 5), they are mutually exclusive.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InterruptMode {
    Mail,
    Dial,
}

pub struct ExtInterrupts {
    pub(crate) syscfg: SYSCFG,
    pub(crate) exti: EXTI,
    pub(crate) nvic: NVIC,
}

static BUTTONS: Mutex<RefCell<Option<(Buttons, fn(Button, &CriticalSection))>>> =
    Mutex::new(RefCell::new(None));

impl Buttons {
    pub(crate) fn split(pg5: Pin<'G', 5>, pg6: Pin<'G', 6>, pc5: Pin<'C', 5>) -> Self {
        Self {
            pin_mail: pg5.into_pull_up_input(),
            pin_back: pg6.into_pull_up_input(),
            pin_dial: pc5.into_pull_up_input(),
        }
    }

    pub fn mail(&self) -> bool {
        self.pin_mail.is_low()
    }

    pub fn back(&self) -> bool {
        self.pin_back.is_low()
    }

    pub fn dial_click(&self) -> bool {
        self.pin_dial.is_low()
    }

    pub fn button(&self, button: Button) -> bool {
        match button {
            Button::Mail => self.mail(),
            Button::Back => self.back(),
            Button::DialClick => self.dial_click(),
        }
    }

    /// Consumes the buttons and registers interrupts to listen for button presses.
    ///
    /// **Note**: not all buttons can be enabled in interrupt mode. See the docs for
    /// [`InterruptMode`] for details.
    ///
    /// The `press_handler` function will be invoked when a button is pressed. The invocation
    /// takes place in a critical section, which can be used to lock [`Mutex`]es from the
    /// [`cortex_m`] crate.
    ///
    /// [`Mutex`]: cortex_m::interrupt::Mutex
    pub fn into_interrupts(
        mut self,
        cfg: &mut ExtInterrupts,
        mode: InterruptMode,
        press_handler: fn(Button, &CriticalSection),
    ) {
        // See note for InterruptMode
        if mode == InterruptMode::Mail {
            self.pin_mail.make_interrupt_source(&mut cfg.syscfg);
            self.pin_mail.trigger_on_edge(&mut cfg.exti, Edge::Falling);
            self.pin_mail.enable_interrupt(&mut cfg.exti);
        }

        if mode == InterruptMode::Dial {
            self.pin_dial.make_interrupt_source(&mut cfg.syscfg);
            self.pin_dial.trigger_on_edge(&mut cfg.exti, Edge::Falling);
            self.pin_dial.enable_interrupt(&mut cfg.exti);
        }

        self.pin_back.make_interrupt_source(&mut cfg.syscfg);
        self.pin_back.trigger_on_edge(&mut cfg.exti, Edge::Falling);
        self.pin_back.enable_interrupt(&mut cfg.exti);

        cortex_m::interrupt::free(|cs| {
            BUTTONS.borrow(cs).replace(Some((self, press_handler)));
        });

        unsafe {
            cfg.nvic.set_priority(interrupt::EXTI9_5, 1);
            NVIC::unmask(interrupt::EXTI9_5);
        }
    }
}

#[interrupt]
fn EXTI9_5() {
    cortex_m::interrupt::free(|cs| {
        let mut st_ref = BUTTONS.borrow(cs).borrow_mut();
        let Some((buttons, handler)) = st_ref.as_mut() else {
            return;
        };
        if buttons.mail() {
            buttons.pin_mail.clear_interrupt_pending_bit();
            handler(Button::Mail, cs);
        }
        if buttons.back() {
            buttons.pin_back.clear_interrupt_pending_bit();
            handler(Button::Back, cs);
        }
        if buttons.dial_click() {
            buttons.pin_dial.clear_interrupt_pending_bit();
            handler(Button::DialClick, cs);
        }
    })
}
