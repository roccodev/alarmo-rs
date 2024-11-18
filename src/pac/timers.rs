use stm32h7xx_hal::{
    gpio::{gpioa, gpiob, gpioc},
    pac::{TIM1, TIM3},
    pwm::{ComplementaryDisabled, ComplementaryImpossible, Pwm, PwmAdvExt},
    rcc::{
        rec::{Tim1, Tim3},
        CoreClocks,
    },
};

pub struct Timers {
    /// TIM1_CH1 (PA8): Dial LED Green channel
    pub tim1_ch1: Pwm<TIM1, 0, ComplementaryDisabled>,
    /// TIM1_CH3 (PA10): Dial LED Red channel
    pub tim1_ch3: Pwm<TIM1, 2, ComplementaryDisabled>,
    /// TIM3_CH3 (PC8): Dial LED Blue channel
    pub tim3_ch3: Pwm<TIM3, 2, ComplementaryImpossible>,
    /// TIM3_CH4 (PB1): LCD backlight
    #[allow(unused)] // difficult to feature-gate
    pub tim3_ch4: Pwm<TIM3, 3, ComplementaryImpossible>,
}

impl Timers {
    pub fn new(
        core_clocks: &CoreClocks,
        tim1: TIM1,
        tim3: TIM3,
        gpioa: gpioa::Parts,
        gpiob: gpiob::Parts,
        gpioc: gpioc::Parts,
        ccdr_tim1: Tim1,
        ccdr_tim3: Tim3,
    ) -> Timers {
        // Dial LED timers
        let (_, (t1c1, t1c3)) = tim1
            .pwm_advanced(
                (gpioa.pa8.into_alternate(), gpioa.pa10.into_alternate()),
                ccdr_tim1,
                core_clocks,
            )
            .prescaler(0)
            .period(u16::MAX)
            .finalize();

        // also includes LCD backlight timer
        let (_, (t3c3, t3c4)) = tim3
            .pwm_advanced(
                (gpiob.pb1.into_alternate(), gpioc.pc8.into_alternate()),
                ccdr_tim3,
                core_clocks,
            )
            .prescaler(0)
            .period(u16::MAX)
            .finalize();

        Timers {
            tim1_ch1: t1c1,
            tim1_ch3: t1c3,
            tim3_ch3: t3c4,
            tim3_ch4: t3c3,
        }
    }
}
