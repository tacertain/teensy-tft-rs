#![no_std]
#![no_main]

use teensy4_panic as _;
use teensy4_bsp::rt::entry;
use teensy4_bsp as bsp;
use bsp::board;

use teensy_tft_rs::{TftDisplay, DoubleBufferedDisplay};

// Teensy 4.1 model identifier for teensy_size tool
// Create as an absolute symbol (ABS section) like the working PlatformIO example
core::arch::global_asm!(
    r#"
    .global _teensy_model_identifier
    .set _teensy_model_identifier, 0x25
    "#
);

#[entry]
fn main() -> ! {
    let mut peripherals = board::t41(board::instances());
    
    // Configure SPI for the display with frequency
    let spi = board::lpspi(
        peripherals.lpspi4,
        board::LpspiPins {
            pcs0: peripherals.pins.p10,  // Chip select pin
            sck: peripherals.pins.p13,   // Clock pin
            sdo: peripherals.pins.p11,   // MOSI pin (Master Out Slave In)
            sdi: peripherals.pins.p12,   // MISO pin (Master In Slave Out)
        },
        30_000_000u32, // 30 MHz SPI frequency for high-speed display updates
    );
    
    // Configure GPIO pins for display control
    let dc_pin = peripherals.gpio2.output(peripherals.pins.p9);
    
    // Configure external LED on pin 2
    let led = peripherals.gpio4.output(peripherals.pins.p2);
    
    // Create base display instance (reset pin is tied to 3V externally)
    let base_display = TftDisplay::new(spi, dc_pin);
    
    // Create double buffered display for smooth graphics
    let mut display = DoubleBufferedDisplay::new(base_display)
        .expect("Failed to create double buffered display");
    
    // Turn on LED to indicate successful initialization
    led.set();
    
    // Counter for LED toggle timing (60 frames = 1 second at 60Hz)
    let mut frame_counter = 0u32;
    
    // Animation variables for smooth movement
    let mut circle_x = 50i32;
    let mut circle_direction = 1i32;
    
    // Main application loop
    loop {
        // Clear the back buffer to black
        display.clear(teensy_tft_rs::graphics::colors::BLACK);
        
        // Draw some graphics using embedded-graphics
        use teensy_tft_rs::graphics::{Graphics, colors::*};
        use embedded_graphics::geometry::{Point, Size};
        
        // Draw a red rectangle
        if let Err(_) = Graphics::draw_filled_rect(&mut display, Point::new(10, 10), Size::new(100, 50), RED) {
            // Handle error
        }
        
        // Draw an animated green circle that bounces horizontally
        if let Err(_) = Graphics::draw_circle(&mut display, Point::new(circle_x, 100), 20, GREEN) {
            // Handle error
        }
        
        // Update circle animation (bounces between x=30 and x=190)
        circle_x += circle_direction * 2; // Move 2 pixels per frame
        if circle_x <= 30 || circle_x >= 190 {
            circle_direction = -circle_direction; // Reverse direction
        }
        
        // Present the frame to the display (swap buffers and transfer)
        if let Err(_) = display.present() {
            // Handle error - could flash LED or take other action
        }
        
        // Increment frame counter
        frame_counter += 1;
        
        // Toggle LED every 60 frames (1 second at 60Hz)
        if frame_counter >= 60 {
            led.toggle();
            frame_counter = 0;
        }
        
        // 60Hz display update delay (~16.67ms at 600MHz)
        cortex_m::asm::delay(10_000_000); // ~16.67ms delay for 60Hz
    }
}
