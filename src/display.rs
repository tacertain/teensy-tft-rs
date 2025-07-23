//! TFT Display Driver Module
//! 
//! Basic display interface for TFT displays

use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayMs;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb565,
};

/// Display error types
#[derive(Debug, Clone, Copy)]
pub enum DisplayError {
    /// SPI communication error
    SpiError,
    /// GPIO pin error
    PinError,
    /// Invalid coordinate
    InvalidCoordinate,
    /// Initialization failed
    InitializationFailed,
}

impl From<&'static str> for DisplayError {
    fn from(_: &'static str) -> Self {
        DisplayError::InitializationFailed
    }
}

/// Simple delay implementation for the display
pub struct Delay;

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        cortex_m::asm::delay(ms as u32 * 600_000); // Approximate delay at 600MHz
    }
}

/// Basic TFT display driver wrapper
pub struct TftDisplay<SPI, DC> {
    spi: SPI,
    dc: DC,
    width: u16,
    height: u16,
}

impl<SPI, DC> TftDisplay<SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    /// Create a new TFT display instance
    /// Reset pin is assumed to be tied to 3V externally
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self {
            spi,
            dc,
            width: 240,
            height: 320,
        }
    }

    /// Initialize the display
    /// Reset pin is connected to 3V externally, so no reset sequence needed
    pub fn init(&mut self) -> Result<(), DisplayError> {
        let mut delay = Delay;
        
        // Reset pin is connected to 3V externally, so no reset sequence needed
        // The display will power up in a reset state
        delay.delay_ms(200); // Wait for display to stabilize after power-on
        
        // Software reset
        self.write_command(0x01).map_err(|_| DisplayError::InitializationFailed)?; // SWRESET
        delay.delay_ms(120);
        
        // Exit sleep mode
        self.write_command(0x11).map_err(|_| DisplayError::InitializationFailed)?; // SLPOUT
        delay.delay_ms(120);
        
        // Display configuration
        self.write_command(0x3A).map_err(|_| DisplayError::InitializationFailed)?; // COLMOD - Pixel Format Set
        self.write_data(&[0x55]).map_err(|_| DisplayError::InitializationFailed)?; // 16-bit RGB565
        
        // Memory Access Control
        self.write_command(0x36).map_err(|_| DisplayError::InitializationFailed)?; // MADCTL
        self.write_data(&[0x48]).map_err(|_| DisplayError::InitializationFailed)?; // MY=0, MX=1, MV=0, ML=0, BGR=1, MH=0
        
        // Display on
        self.write_command(0x29).map_err(|_| DisplayError::InitializationFailed)?; // DISPON
        delay.delay_ms(100);
        
        Ok(())
    }

    /// Clear the display to black
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        // Use optimized fill_rect for better performance
        self.fill_rect(0, 0, self.width, self.height, Rgb565::BLACK)
    }

    /// Get display dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Write a command to the display
    pub fn write_command(&mut self, command: u8) -> Result<(), &'static str> {
        self.dc.set_low().map_err(|_| "DC pin error")?;
        self.spi.write(&[command]).map_err(|_| "SPI write error")?;
        Ok(())
    }

    /// Write data to the display
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), &'static str> {
        self.dc.set_high().map_err(|_| "DC pin error")?;
        self.spi.write(data).map_err(|_| "SPI write error")?;
        Ok(())
    }
    
    /// Set the drawing window for pixel operations
    fn set_address_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), &'static str> {
        // Column address set
        self.write_command(0x2A)?; // CASET
        self.write_data(&[
            (x0 >> 8) as u8, (x0 & 0xFF) as u8,
            (x1 >> 8) as u8, (x1 & 0xFF) as u8,
        ])?;
        
        // Page address set  
        self.write_command(0x2B)?; // PASET
        self.write_data(&[
            (y0 >> 8) as u8, (y0 & 0xFF) as u8,
            (y1 >> 8) as u8, (y1 & 0xFF) as u8,
        ])?;
        
        // Memory write command
        self.write_command(0x2C)?; // RAMWR
        Ok(())
    }
    
    /// Fill a rectangular area with a single color (optimized bulk transfer)
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565) -> Result<(), DisplayError> {
        // Validate bounds
        if x >= self.width || y >= self.height {
            return Err(DisplayError::InvalidCoordinate);
        }
        
        // Clip to display bounds
        let end_x = (x + w).min(self.width);
        let end_y = (y + h).min(self.height);
        
        if end_x <= x || end_y <= y {
            return Ok(()); // Nothing to draw
        }
        
        // Set drawing window
        self.set_address_window(x, y, end_x - 1, end_y - 1)
            .map_err(|_| DisplayError::SpiError)?;
        
        // Prepare color data
        let color_value = color.into_storage();
        let color_bytes = [
            (color_value >> 8) as u8,
            (color_value & 0xFF) as u8,
        ];
        
        // Fill the rectangle
        let pixel_count = (end_x - x) as u32 * (end_y - y) as u32;
        for _ in 0..pixel_count {
            self.write_data(&color_bytes).map_err(|_| DisplayError::SpiError)?;
        }
        
        Ok(())
    }
}

// Basic DrawTarget implementation for embedded-graphics
impl<SPI, DC> DrawTarget for TftDisplay<SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(coord, color) = pixel;
            
            // Validate coordinates
            if coord.x < 0 || coord.y < 0 || coord.x >= self.width as i32 || coord.y >= self.height as i32 {
                continue; // Skip pixels outside display bounds
            }
            
            // Set drawing window to single pixel
            self.set_address_window(
                coord.x as u16, 
                coord.y as u16,
                coord.x as u16, 
                coord.y as u16
            ).map_err(|_| DisplayError::SpiError)?;
            
            // Write pixel data (RGB565 format)
            let color_value = color.into_storage();
            let color_bytes = [
                (color_value >> 8) as u8,
                (color_value & 0xFF) as u8,
            ];
            self.write_data(&color_bytes).map_err(|_| DisplayError::SpiError)?;
        }
        Ok(())
    }
}

impl<SPI, DC> OriginDimensions for TftDisplay<SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
