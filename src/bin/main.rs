#![no_std]
#![no_main]

use teensy4_panic as _;
use teensy4_bsp::rt::entry;
use teensy4_bsp as bsp;
use bsp::board;

use teensy_tft_rs::TftDisplay;

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
        1_000_000u32, // 1 MHz SPI frequency
    );
    
    // Configure GPIO pins for display control
    let dc_pin = peripherals.gpio2.output(peripherals.pins.p9);
    
    // Configure external LED on pin 2
    let led = peripherals.gpio4.output(peripherals.pins.p2);
    
    // Create display instance (reset pin is tied to 3V externally)
    let mut display = TftDisplay::new(spi, dc_pin);
    
    // Initialize the display
    display.init().expect("Failed to initialize display");
    
    // Turn on LED to indicate successful initialization
    led.set();
    
    // Counter for LED toggle timing (60 frames = 1 second at 60Hz)
    let mut frame_counter = 0u32;
    
    // Main application loop
    loop {
        // Clear the display
        if let Err(_) = display.clear() {
            // Handle error
        }
        
        // Draw some graphics using embedded-graphics
        use teensy_tft_rs::graphics::{Graphics, colors::*};
        use embedded_graphics::geometry::{Point, Size};
        
        // Draw a red rectangle
        if let Err(_) = Graphics::draw_filled_rect(&mut display, Point::new(10, 10), Size::new(100, 50), RED) {
            // Handle error
        }
        
        // Draw a green circle
        if let Err(_) = Graphics::draw_circle(&mut display, Point::new(150, 100), 30, GREEN) {
            // Handle error
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
