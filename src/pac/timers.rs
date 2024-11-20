use stm32h7xx_hal::{
    gpio::Pin,
    pac::{TIM1, TIM3},
    pwm::{ComplementaryDisabled, ComplementaryImpossible, Pwm, PwmAdvExt},
    rcc::{
        rec::{Tim1, Tim3},
        CoreClocks,
    },
};

pub struct DialTimers {
    /// TIM1_CH1 (PA8): Dial LED Green channel
    pub tim1_ch1: Pwm<TIM1, 0, ComplementaryDisabled>,
    /// TIM1_CH3 (PA10): Dial LED Red channel
    pub tim1_ch3: Pwm<TIM1, 2, ComplementaryDisabled>,
    /// TIM3_CH3 (PC8): Dial LED Blue channel
    pub tim3_ch3: Pwm<TIM3, 2, ComplementaryImpossible>,
    // + TIM3_CH4 (PB1): LCD backlight -> moved to display module
}

pub fn split_timers(
    core_clocks: &CoreClocks,
    tim1: TIM1,
    tim3: TIM3,
    pa8: Pin<'A', 8>,
    pa10: Pin<'A', 10>,
    pb1: Pin<'B', 1>,
    pc8: Pin<'C', 8>,
    ccdr_tim1: Tim1,
    ccdr_tim3: Tim3,
) -> (DialTimers, Pwm<TIM3, 3, ComplementaryImpossible>) {
    // Dial LED timers
    let (_, (t1c1, t1c3)) = tim1
        .pwm_advanced(
            (pa8.into_alternate(), pa10.into_alternate()),
            ccdr_tim1,
            core_clocks,
        )
        .prescaler(0)
        .period(u16::MAX)
        .finalize();

    // also includes LCD backlight timer
    let (t3c4, t3c3) = display_timer(core_clocks, tim3, pb1, pc8, ccdr_tim3);

    (
        DialTimers {
            tim1_ch1: t1c1,
            tim1_ch3: t1c3,
            tim3_ch3: t3c3,
        },
        t3c4,
    )
}

pub fn display_timer(
    core_clocks: &CoreClocks,
    tim3: TIM3,
    pb1: Pin<'B', 1>,
    pc8: Pin<'C', 8>,
    ccdr_tim3: Tim3,
) -> (
    Pwm<TIM3, 3, ComplementaryImpossible>,
    Pwm<TIM3, 2, ComplementaryImpossible>,
) {
    // LCD backlight timer
    let (_, (t3c4, t3c3)) = tim3
        .pwm_advanced(
            (pb1.into_alternate(), pc8.into_alternate()),
            ccdr_tim3,
            core_clocks,
        )
        .prescaler(0)
        .period(u16::MAX)
        .finalize();
    (t3c4, t3c3)
}
