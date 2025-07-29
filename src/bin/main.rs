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

// Static frame buffers for double buffering
// Full size buffers (240x320 = 76,800 u16 = 153.6KB each, 307.2KB total)
static mut BACK_BUFFER: [u16; 240 * 320] = [0; 240 * 320];
static mut FRONT_BUFFER: [u16; 240 * 320] = [0; 240 * 320];

#[entry]
fn main() -> ! {
    let mut peripherals = board::t41(board::instances());
        
    let dma = peripherals.dma[0].take().unwrap();

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
    
    // Turn on LED immediately to show we got this far
    led.set();

    // Test 1: Basic display creation (without double buffering)
    cortex_m::asm::delay(120_000_000); // Initial delay
    
    // Create base display instance (reset pin is tied to 3V externally)
    let base_display = TftDisplay::new(spi, dc_pin);
    
    // Flash LED to indicate display creation succeeded
    led.clear();
    cortex_m::asm::delay(60_000_000); // ~100ms delay
    led.set();
    cortex_m::asm::delay(60_000_000); // ~100ms delay

    // Test 2: Now try double buffered display creation with static buffers
    let mut display = unsafe {
        DoubleBufferedDisplay::new(
            base_display, 
            spi,
            dma,
            &mut BACK_BUFFER, 
            &mut FRONT_BUFFER
        ).expect("Failed to create double buffered display")
    };
    
    // Flash LED twice to indicate double buffering succeeded
    led.clear();
    cortex_m::asm::delay(60_000_000);
    led.set();
    cortex_m::asm::delay(60_000_000);
    led.clear();
    cortex_m::asm::delay(60_000_000);
    led.set();
    
    // Small delay before starting graphics
    cortex_m::asm::delay(120_000_000); // ~200ms delay
    
    // Counter for LED toggle timing (60 frames = 1 second at 60Hz)
    let mut frame_counter = 0u32;
    
    // Animation variables for smooth movement  
    let mut circle_x = 120i32; // Start in center
    let mut circle_direction = 1i32;
    
    // Main application loop with graphics
    loop {
        // Clear the back buffer to black
        display.clear(teensy_tft_rs::graphics::colors::BLACK);
        
        // Draw some graphics using embedded-graphics
        use teensy_tft_rs::graphics::{Graphics, colors::*};
        use embedded_graphics::geometry::{Point, Size};
        
        // Draw a red rectangle (larger for full screen)
        if let Err(_) = Graphics::draw_filled_rect(&mut display, Point::new(20, 20), Size::new(100, 60), RED) {
            // Handle error
        }
        
        // Draw an animated green circle that bounces horizontally across full width
        if let Err(_) = Graphics::draw_circle(&mut display, Point::new(circle_x, 160), 20, GREEN) {
            // Handle error
        }
        
        // Update circle animation (bounces between x=30 and x=210 for full screen)
        circle_x += circle_direction * 2; // Move 2 pixels per frame
        if circle_x <= 30 || circle_x >= 210 {
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
