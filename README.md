# Rust on Alarmo!
[![Crates.io Version](https://img.shields.io/crates/v/alarmo)](https://crates.io/crates/alarmo)
[![docs.rs (with version)](https://img.shields.io/docsrs/alarmo/latest)](https://docs.rs/alarmo)

This project provides a convenient API to bootstrap firmware and access peripherals on the Nintendo Alarmo, using Rust.

## Status

This started as a Rust port of GaryOderNichts's MIT-licensed [alarmo](https://github.com/GaryOderNichts/alarmo) repository, which uses the [official hardware abstraction layer](https://github.com/STMicroelectronics/STM32CubeH7). As a proof-of-concept, I first made a Rust loader that would just call GaryOderNichts's implementation, then I incrementally rewrote each part in the Rust module.

Consider this list of goals tentative:

- [x] It works!
- [x] LCD frontend using the `display_interface` crate
- [ ] Better build environment (automate `objcopy` and firmware signing)
- [ ] Button inputs
- [ ] Dial input
- [x] Dial LED
- [ ] Allocator with extended RAM

As a general goal, I'd like to use Rust-native HALs (e.g. [stm32h7xx-hal](https://crates.io/crates/stm32h7xx-hal)) whenever possible. In its current state, the project still depends on the official HAL, for which it generates bindings using `bindgen`.

## Usage

### Running an example

1. Compile the example, for example `lcd`:
```sh
# "--features display" required to build the LCD example
cargo build --example lcd --features display
```
2. Convert the example ELF into BINF:
```
arm-none-eabi-objcopy target/thumbv7em-none-eabihf/debug/examples/lcd lcd.bin
```
3. Sign the firmware to get the `a.bin`.

### Creating a project

1. After you've created the Cargo project, copy the `link.ld` file and `.cargo/` directory from this repository to the root of your crate.
2. Add the library as a dependency:
```sh
# With LCD support
cargo add alarmo -F display

# No LCD support
cargo add alarmo

# Optionally, add some other dependencies for peripherals, see examples for details
# cargo add mipidsi
# cargo add embedded-graphics
```
3. Build the project:
```
cargo build
```
4. Convert the result ELF into BINF (change `debug` to `release` for `--release`):
```
arm-none-eabi-objcopy target/thumbv7em-none-eabihf/debug/your_crate your_crate.bin
```
3. Sign the firmware to get the `a.bin`.

## License
The library and its examples are dual-licensed under both [Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT). Please also refer to the [license documentation](https://github.com/STMicroelectronics/STM32CubeH7/blob/master/LICENSE.md) for the official HAL.

## Credits
* [GaryOderNichts](https://github.com/GaryOderNichts/), whose [project](https://github.com/GaryOderNichts/alarmo) this repository is derived from, and for their [blog post](https://garyodernichts.blogspot.com/2024/10/looking-into-nintendo-alarmo.html) exploring the Alarmo.
* [Spinda](https://spinda.net/) and [hexkyz](https://twitter.com/hexkyz) for their incredible work reverse-engineering the Alarmo!