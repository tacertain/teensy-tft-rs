#![no_std]
#![no_main]

use teensy4_panic as _;
use teensy4_bsp::rt::entry;
use teensy4_bsp as bsp;
use bsp::board;

use teensy_tft_rs::TftDisplay;

#[entry]
fn main() -> ! {
    let mut peripherals = board::t40(board::instances());
    
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
    let reset_pin = peripherals.gpio2.output(peripherals.pins.p8);
    
    // Create display instance
    let mut display = TftDisplay::new(spi, dc_pin, reset_pin);
    
    // Initialize the display
    if let Err(_) = display.init() {
        // Handle initialization error
        loop {}
    }
    
    // Main application loop
    loop {
        // Clear the display
        if let Err(_) = display.clear() {
            // Handle error
        }
        
        // Draw some graphics
        // Note: This is a simplified example
        // Real implementation would need proper display driver integration
        
        // Add a small delay
        cortex_m::asm::delay(1_000_000);
    }
}
