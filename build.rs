use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=vendor");

    let bindings = bindgen::Builder::default()
        .use_core()
        .header("vendor/wrapper.h")
        .clang_arg("-Ivendor")
        .clang_arg("-ISTM32CubeH7/Drivers/CMSIS/Include")
        .clang_arg("-ISTM32CubeH7/Drivers/CMSIS/Device/ST/STM32H7xx/Include")
        .clang_arg("-ISTM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Inc")
        .clang_arg("-I/usr/arm-none-eabi/include") // TODO
        .clang_arg("-DSTM32H730xx")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .wrap_static_fns(true)
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Don't compile HAL on docs.rs
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if arch == "arm" {
        cc::Build::new()
            .files([
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_cortex.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_dma_ex.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_dma.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_dma.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_gpio.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_gpio.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_mdma.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_mdma.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_pwr_ex.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_pwr.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_pwr.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_rcc_ex.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_rcc.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_rcc.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_sram.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_fmc.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_tim_ex.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal_tim.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_ll_tim.c",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Src/stm32h7xx_hal.c",
                "vendor/guard.c",
                "vendor/startup_stm32h730xx.s",
            ])
            .includes([
                "STM32CubeH7/Drivers/CMSIS/Include",
                "STM32CubeH7/Drivers/CMSIS/Device/ST/STM32H7xx/Include",
                "STM32CubeH7/Drivers/STM32H7xx_HAL_Driver/Inc",
                "vendor",
            ])
            .flag("-DSTM32H730xx")
            .flag("-mthumb")
            .flag("-march=armv7e-m")
            .flag("-mtune=cortex-m7")
            .flag("-mfpu=fpv5-d16")
            .flag("-mfloat-abi=hard")
            .flag("-O0") // TODO figure out why optimizations break it
            .compile("vendor");
    } else {
        println!("cargo::warning=Not compiling HAL on wrong arch {arch}");
    }
}
