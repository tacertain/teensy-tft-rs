#![no_std]
#![no_main]

use teensy4_panic as _;
use teensy4_bsp::rt::entry;
use teensy4_bsp as bsp;
use bsp::board;

use teensy_tft_rs::dma_display::{DmaTftDisplay, DmaDoubleBufferedDisplay};

// For the DMA async example - using pin_utils for pinning futures
use pin_utils::pin_mut;

// Teensy 4.1 model identifier for teensy_size tool
core::arch::global_asm!(
    r#"
    .global _teensy_model_identifier
    .set _teensy_model_identifier, 0x25
    "#
);

// Static frame buffers for double buffering
static mut BACK_BUFFER: [u16; 240 * 320] = [0; 240 * 320];
static mut FRONT_BUFFER: [u16; 240 * 320] = [0; 240 * 320];

#[entry]
fn main() -> ! {
    let mut peripherals = board::t41(board::instances());
    
    // Configure DMA channels for SPI transfers
    let mut dma = peripherals.dma;
    let mut dma_channel_tx = dma[0].take().unwrap(); // Use DMA channel 0 for TX
    dma_channel_tx.set_disable_on_completion(true);
    
    let mut dma_channel_rx = dma[1].take().unwrap(); // Use DMA channel 1 for RX
    dma_channel_rx.set_disable_on_completion(true);
    
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

    // Initial delay
    cortex_m::asm::delay(120_000_000);
    
    // Create TFT display instance with DMA support
    let base_display = DmaTftDisplay::new_with_dma(spi, dc_pin, dma_channel_tx, dma_channel_rx);
    
    // Flash LED to indicate display creation succeeded
    led.clear();
    cortex_m::asm::delay(60_000_000); // ~100ms delay
    led.set();
    cortex_m::asm::delay(60_000_000); // ~100ms delay

    // Create DMA-enabled double buffered display with static buffers
    let mut display = unsafe {
        DmaDoubleBufferedDisplay::new(
            base_display, 
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
    
    // Main application loop with DMA graphics
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
        
        // Variable to track whether DMA was used for LED indication
        let used_dma;
        
        // Use DMA-optimized present method for demonstration
        // In the current implementation, this falls back to blocking but uses larger chunk sizes
        if frame_counter % 10 == 0 {
            // Every 10th frame, demonstrate the async DMA interface
            // This would be where DMA acceleration happens with proper hardware support
            let async_present = display.present_async();
            pin_mut!(async_present);
            
            // For now, we'll use the fact that our "async" method completes immediately
            // In a real DMA implementation, this would queue the transfer and return immediately
            use core::task::{Context, Poll, Waker};
            use core::future::Future;
            
            // Simple polling to completion (simulates blocking on DMA completion)
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);
            
            loop {
                match async_present.as_mut().poll(&mut context) {
                    Poll::Ready(Ok(result)) => {
                        used_dma = result.used_dma;
                        break;
                    }
                    Poll::Ready(Err(_)) => {
                        // Handle error
                        used_dma = false;
                        break;
                    }
                    Poll::Pending => {
                        // In a real DMA implementation, we'd yield here
                        // For now, continue polling
                        continue;
                    }
                }
            }
        } else {
            // Regular blocking present for other frames
            if let Err(_) = display.present() {
                // Handle error - could flash LED or take other action
            }
            used_dma = false; // Blocking write doesn't use DMA
        }
        
        // Set LED based on whether DMA was used
        // LED ON = DMA transfers being used
        // LED OFF = Blocking writes being used
        if used_dma {
            led.set(); // Turn LED on for DMA
        } else {
            led.clear(); // Turn LED off for blocking
        }
        
        // Increment frame counter
        frame_counter += 1;
        
        // 60Hz display update delay (~16.67ms at 600MHz)
        cortex_m::asm::delay(10_000_000); // ~16.67ms delay for 60Hz
    }
}
