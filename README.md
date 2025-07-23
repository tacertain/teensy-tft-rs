# Teensy TFT Rust

An embedded Rust library and application for controlling TFT displays on Teensy 4.0/4.1 microcontrollers.

## Overview

This project provides a `no_std` Rust library for interfacing with TFT displays using the Teensy 4.x platform. It includes:

- Low-level display drivers using SPI communication
- Graphics primitives and drawing utilities via `embedded-graphics`
- Hardware abstraction layer (HAL) integration
- Example applications demonstrating usage

## Hardware Requirements

- **Teensy 4.0 or 4.1** microcontroller
- **TFT Display** with SPI interface (e.g., ILI9341, ST7735)
- **Connections:**
  - MOSI (Pin 11) → Display SDI/SDA
  - SCK (Pin 13) → Display SCK
  - CS (Pin 10) → Display CS
  - DC (Pin 9) → Display DC/RS
  - RST (Pin 8) → Display Reset

## Software Requirements

### Rust Toolchain
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM Cortex-M target
rustup target add thumbv7em-none-eabihf
```

### Development Tools
```bash
# Install required tools
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Install Teensy Loader CLI (for flashing)
# Download from: https://www.pjrc.com/teensy/loader_cli.html
```

## Building

### Library
```bash
# Build the library
cargo build --release

# Check for compilation errors
cargo check
```

### Main Application
```bash
# Build the main binary
cargo build --release --bin main

# Flash to Teensy (requires teensy_loader_cli)
cargo run --release --bin main
```

## Project Structure

```
teensy-tft-rs/
├── src/
│   ├── lib.rs              # Library root
│   ├── display.rs          # TFT display drivers
│   ├── graphics.rs         # Graphics utilities
│   └── bin/
│       └── main.rs         # Main application
├── .cargo/
│   └── config.toml         # Cargo configuration
├── .github/
│   └── copilot-instructions.md
├── memory.x                # Memory layout
├── Cargo.toml             # Dependencies and metadata
└── README.md
```

## Usage Example

```rust
#![no_std]
#![no_main]

use teensy4_panic as _;
use cortex_m_rt::entry;
use teensy4_bsp as bsp;
use bsp::board;

use teensy_tft_rs::{TftDisplay, Graphics, colors};

#[entry]
fn main() -> ! {
    let mut peripherals = board::init();
    
    // Initialize SPI and GPIO pins
    let spi = board::lpspi(/* ... */);
    let dc_pin = peripherals.pins.p9.into_push_pull_output();
    let reset_pin = peripherals.pins.p8.into_push_pull_output();
    
    // Create and initialize display
    let mut display = TftDisplay::new(spi, dc_pin, reset_pin);
    display.init().unwrap();
    
    loop {
        // Your application code here
        display.clear().unwrap();
        
        // Draw graphics using embedded-graphics
        Graphics::draw_filled_rect(
            &mut display,
            Point::new(10, 10),
            Size::new(100, 50),
            colors::RED
        ).unwrap();
        
        cortex_m::asm::delay(1_000_000);
    }
}
```

## Features

- **Hardware Abstraction**: Uses `embedded-hal` traits for portability
- **Graphics Support**: Integration with `embedded-graphics` library
- **Memory Efficient**: `no_std` design with static allocation
- **Real-time**: Suitable for real-time applications
- **Modular**: Separate display drivers and graphics utilities

## Development

### VS Code Extensions
This workspace includes configuration for:
- **Rust Analyzer**: Advanced Rust language support
- **Crates**: Dependency management assistance

### Debugging
- Use an external debugger like ST-Link or J-Link
- Configure OpenOCD or similar debugging tools
- Set breakpoints and inspect memory in VS Code

### Testing
```bash
# Run unit tests (requires std environment)
cargo test

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy
```

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Ensure all dependencies are compatible with `no_std`
2. **Flashing Issues**: Check that `teensy_loader_cli` is in your PATH
3. **Display Not Working**: Verify SPI connections and initialization sequence
4. **Memory Issues**: Check `memory.x` layout matches your Teensy model

### Getting Help

- Check the [embedded-hal documentation](https://docs.rs/embedded-hal/)
- Review [embedded-graphics examples](https://docs.rs/embedded-graphics/)
- Visit the [Teensy Forum](https://forum.pjrc.com/) for hardware-specific questions

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Changelog

### v0.1.0
- Initial project setup
- Basic TFT display driver structure
- Graphics utilities foundation
- Example main application
