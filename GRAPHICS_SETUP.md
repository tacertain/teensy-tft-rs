# ILI9341 Graphics Library Configuration

This document describes the graphics library configuration for the Teensy 4.1 TFT project.

## Overview

The project has been configured with a basic graphics library setup that includes:

- **TFT Display Driver**: Basic driver wrapper for ILI9341-compatible displays
- **embedded-graphics Integration**: Full support for 2D graphics primitives
- **Teensy 4.1 SPI Configuration**: Proper pin assignments for TFT communication

## Hardware Configuration

### Teensy 4.1 Pin Assignments
- **SPI4**: LPSPI4 peripheral for display communication
- **DC Pin**: GPIO pin 8 (Data/Command control)
- **Reset Pin**: GPIO pin 9 (Display reset)
- **SPI Pins**: Standard LPSPI4 pins (MOSI, SCK)

### Display Specifications
- **Resolution**: 240x320 pixels
- **Color Format**: RGB565 (16-bit color)
- **Interface**: SPI with DC and Reset control

## Software Architecture

### Display Module (`src/display.rs`)
```rust
pub struct TftDisplay<SPI, DC, RST> {
    spi: SPI,
    dc: DC,
    reset: RST,
    width: u16,
    height: u16,
}
```

Key features:
- Generic over SPI, DC, and Reset pin types
- embedded-graphics `DrawTarget` trait implementation
- Basic initialization and control methods
- Support for 240x320 pixel displays

### Graphics Module (`src/graphics.rs`)
```rust
pub struct Graphics;
```

Provides utility functions:
- `draw_filled_rect()`: Draw filled rectangles
- `draw_circle()`: Draw outlined circles
- `draw_text()`: Text rendering (placeholder)

Color palette includes: BLACK, WHITE, RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN

### Main Application (`src/bin/main.rs`)
Basic application loop that:
1. Initializes the Teensy 4.1 board
2. Configures SPI and GPIO pins
3. Creates and initializes the display
4. Draws graphics primitives in a loop

## Dependencies

```toml
[dependencies]
teensy4-bsp = { version = "0.5", features = ["rt"] }
embedded-graphics = "0.8"
display-interface = "0.4"
display-interface-spi = "0.4"
ili9341 = "0.5"
```

## Current Status

✅ **Completed:**
- Basic display driver structure
- embedded-graphics integration
- Teensy 4.1 hardware configuration
- SPI communication setup
- Build system configuration

🔄 **In Progress:**
- Full ILI9341 initialization sequence
- Pixel-level drawing implementation
- Hardware testing and validation

## Next Steps

1. **Complete ILI9341 Integration**: Implement full display initialization and pixel drawing
2. **Hardware Testing**: Test with actual Teensy 4.1 and ILI9341 display
3. **Performance Optimization**: Optimize SPI communication and drawing routines
4. **Advanced Graphics**: Add text rendering, image support, and animations

## Usage Example

```rust
use teensy_tft_rs::display::TftDisplay;
use teensy_tft_rs::graphics::{Graphics, colors::*};
use embedded_graphics::geometry::{Point, Size};

// Initialize display
let mut display = TftDisplay::new(spi, dc_pin, reset_pin);
display.init().expect("Failed to initialize display");

// Draw graphics
Graphics::draw_filled_rect(&mut display, Point::new(10, 10), Size::new(100, 50), RED);
Graphics::draw_circle(&mut display, Point::new(150, 100), 30, GREEN);
```

## Build Instructions

```powershell
# Build for Teensy 4.1
cargo build --release

# The output will be in:
# target/thumbv7em-none-eabihf/release/main
```

The project is now ready for hardware testing and further development!
