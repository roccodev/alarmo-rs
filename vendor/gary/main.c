#include "main.h"
#include <stm32h7xx_hal.h>
#include <stm32h7xx_ll_rcc.h>
#include "lcd.h"

#include "cat.inc"

SRAM_HandleTypeDef fmcHandle;
TIM_HandleTypeDef tim3Handle;

int GaryMain(TIM_HandleTypeDef *timHandle)
{
    tim3Handle = *timHandle;

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
