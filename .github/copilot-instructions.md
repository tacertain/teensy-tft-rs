<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Teensy TFT Rust Project Instructions

This is an embedded Rust project for Teensy 4.1 microcontrollers with TFT display support.

## Project Context
- Target: Teensy 4.1 (ARM Cortex-M7, iMXRT1062)
- Architecture: ARM Cortex-M7 (thumbv7em-none-eabihf)
- No standard library (`#![no_std]`)
- Uses embedded-hal traits for hardware abstraction
- TFT display communication via SPI

## Key Dependencies
- `cortex-m` and `cortex-m-rt`: ARM Cortex-M runtime
- `teensy4-bsp`: Teensy 4.x board support package
- `embedded-hal`: Hardware abstraction layer
- `embedded-graphics`: 2D graphics library
- `display-interface-spi`: SPI display interface

## Coding Guidelines
- All code must be `#![no_std]` compatible
- Use `Result<T, E>` for error handling (no `std::error::Error`)
- Prefer `heapless` collections over `std` collections
- Use `nb` for non-blocking APIs
- Memory management must be static or stack-based (no heap allocation)
- Interrupt handlers should be minimal and fast
- Use `cortex_m::interrupt::free()` for critical sections

## Hardware Considerations
- SPI communication for display
- GPIO pins for display control (DC, Reset)
- Memory layout defined in `memory.x`
- Real-time constraints for display updates
- Power management considerations

## Testing
- Use `#[cfg(test)]` with `std` feature for unit tests
- Hardware-in-the-loop testing on actual Teensy hardware
- Simulation testing where possible
