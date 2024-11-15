#include "main.h"
#include <stm32h7xx_hal.h>
#include <stm32h7xx_ll_rcc.h>
// #include "system_stm32h7xx.c"
// #include "stm32h7xx_hal_msp.c"
// #include "stm32h7xx_it.c"
// #include "string.c"
#include "lcd.h"

#include "cat.inc"

static void FMC_Init(void);
static void TIM3_Init(void);

SRAM_HandleTypeDef fmcHandle;
TIM_HandleTypeDef tim3Handle;


static void TIMx_PWM_MspInit(TIM_HandleTypeDef *handle)
{
    GPIO_InitTypeDef gpioConfig = { 0 };

    if (handle->Instance == TIM3) {
        __HAL_RCC_GPIOB_CLK_ENABLE();
        __HAL_RCC_GPIOC_CLK_ENABLE();

        gpioConfig.Alternate = GPIO_AF2_TIM3;
        gpioConfig.Pull = GPIO_NOPULL;
        gpioConfig.Speed = GPIO_SPEED_FREQ_LOW;
        gpioConfig.Pin = GPIO_PIN_1;
        gpioConfig.Mode = GPIO_MODE_AF_PP;
        HAL_GPIO_Init(GPIOB, &gpioConfig);

        gpioConfig.Pin = GPIO_PIN_8;
        gpioConfig.Alternate = GPIO_AF2_TIM3;
        gpioConfig.Speed = GPIO_SPEED_FREQ_LOW;
        gpioConfig.Pull = GPIO_NOPULL;
        gpioConfig.Mode = GPIO_MODE_AF_PP;
        HAL_GPIO_Init(GPIOC, &gpioConfig);
    }
}

int GaryMain(TIM_HandleTypeDef *timHandle)
{
    tim3Handle = *timHandle;
    TIMx_PWM_MspInit(&tim3Handle);

    // Initialize the LCD
    LCD_Init();

    uint32_t lcdId = 0;
    LCD_RDID(&lcdId);

    // Setup backlight
    LCD_SetBrightness(1.0f);

    // Clear screen
    LCD_DrawRect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, 255, 0, 0);

    // Draw cat
    LCD_DrawScreenBuffer(image_data, sizeof(image_data));

    // Turn on the display
    LCD_DISPON();

    while (1)
        ;
}

static void FMC_Init(void)
{
    FMC_NORSRAM_TimingTypeDef timing;

    fmcHandle.Instance = FMC_Bank1_R;
    fmcHandle.Extended = FMC_Bank1E_R;

    fmcHandle.Init.WaitSignalActive   = 0;
    fmcHandle.Init.WriteOperation     = FMC_WRITE_OPERATION_ENABLE;
    fmcHandle.Init.NSBank             = FMC_NORSRAM_BANK1;
    fmcHandle.Init.MemoryDataWidth    = FMC_NORSRAM_MEM_BUS_WIDTH_16;
    fmcHandle.Init.BurstAccessMode    = FMC_BURST_ACCESS_MODE_DISABLE;
    fmcHandle.Init.DataAddressMux     = FMC_DATA_ADDRESS_MUX_DISABLE;
    fmcHandle.Init.MemoryType         = FMC_MEMORY_TYPE_SRAM;
    fmcHandle.Init.WaitSignalPolarity = FMC_WAIT_SIGNAL_POLARITY_LOW;

    timing.BusTurnAroundDuration = 0;
    timing.CLKDivision           = 1;
    timing.DataLatency           = 0;
    timing.AccessMode            = 0;
    timing.DataSetupTime         = 2;
    timing.AddressSetupTime      = 2;   
    timing.AddressHoldTime       = 0;
    if (HAL_SRAM_Init(&fmcHandle, &timing, NULL) != HAL_OK) {
        while (1)
            ;
    }

    HAL_SetFMCMemorySwappingConfig(FMC_SWAPBMAP_SDRAM_SRAM);
}


static void TIM3_Init(void)
{
    TIM_MasterConfigTypeDef masterConfig = { 0 };
    TIM_OC_InitTypeDef channelConfig = { 0 };

    tim3Handle.Instance = TIM3;
    tim3Handle.Init.AutoReloadPreload = TIM_AUTORELOAD_PRELOAD_DISABLE;
    tim3Handle.Init.Prescaler         = 0;
    tim3Handle.Init.CounterMode       = TIM_COUNTERMODE_UP;
    tim3Handle.Init.Period            = 0xffff;
    tim3Handle.Init.ClockDivision     = TIM_CLOCKDIVISION_DIV1;
    if (HAL_TIM_PWM_Init(&tim3Handle) != HAL_OK) {
        while (1)
            ;
    }

    masterConfig.MasterSlaveMode     = TIM_MASTERSLAVEMODE_DISABLE;
    masterConfig.MasterOutputTrigger = TIM_TRGO_RESET;
    if (HAL_TIMEx_MasterConfigSynchronization(&tim3Handle, &masterConfig) != HAL_OK) {
        while (1)
            ;
    }

    channelConfig.OCFastMode = TIM_OCFAST_DISABLE;
    channelConfig.Pulse      = 0;
    channelConfig.OCPolarity = TIM_OCPOLARITY_HIGH;
    channelConfig.OCMode     = TIM_OCMODE_PWM1;
    if (HAL_TIM_PWM_ConfigChannel(&tim3Handle, &channelConfig, TIM_CHANNEL_3) != HAL_OK) {
        while (1)
            ;
    }

    if (HAL_TIM_PWM_ConfigChannel(&tim3Handle, &channelConfig, TIM_CHANNEL_4) != HAL_OK) {
        while (1)
            ;
    }

    TIMx_PWM_MspInit(&tim3Handle);
}
