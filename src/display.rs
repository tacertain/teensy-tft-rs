//! TFT Display Driver Module
//! 
//! Basic display interface for TFT displays with double buffering support

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

/// Double-buffered display for smooth graphics rendering
/// 
/// This implementation uses two frame buffers - one for drawing (back buffer)
/// and one for display (front buffer). Drawing operations write to the back buffer,
/// and present() swaps the buffers and transfers to the physical display.
/// 
/// Buffers are provided externally to allow static allocation in embedded systems.
pub struct DoubleBufferedDisplay<'a, SPI, DC> {
    display: TftDisplay<SPI, DC>,
    // Frame buffers provided externally - RGB565 format (2 bytes per pixel)
    back_buffer: &'a mut [u16],
    front_buffer: &'a mut [u16],
    width: u16,
    height: u16,
    dirty: bool, // Track if back buffer has changes
}

impl<'a, SPI, DC> DoubleBufferedDisplay<'a, SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    /// Create a new double-buffered display with externally provided buffers
    /// 
    /// # Arguments
    /// * `display` - The underlying TFT display
    /// * `back_buffer` - Mutable slice for the back buffer (drawing buffer)
    /// * `front_buffer` - Mutable slice for the front buffer (display buffer)
    /// 
    /// Both buffers should be the same size and match the display resolution.
    /// For a 240x320 display, each buffer should be 76,800 u16 elements.
    pub fn new(
        mut display: TftDisplay<SPI, DC>,
        back_buffer: &'a mut [u16],
        front_buffer: &'a mut [u16],
    ) -> Result<Self, DisplayError> {
        // Validate buffer sizes match
        if back_buffer.len() != front_buffer.len() {
            return Err(DisplayError::InitializationFailed);
        }
        
        // Initialize the underlying display
        display.init()?;
        
        let (width, height) = display.dimensions();
        
        // Validate buffer size matches display dimensions
        let expected_size = (width as usize) * (height as usize);
        if back_buffer.len() != expected_size {
            return Err(DisplayError::InitializationFailed);
        }
        
        // Clear the buffers
        for pixel in back_buffer.iter_mut() {
            *pixel = 0; // Black (RGB565 0x0000)
        }
        for pixel in front_buffer.iter_mut() {
            *pixel = 0; // Black (RGB565 0x0000)
        }
        
        let mut double_buffered = Self {
            display,
            back_buffer,
            front_buffer,
            width,
            height,
            dirty: false,
        };
        
        // Clear the physical display
        double_buffered.display.clear()?;
        
        Ok(double_buffered)
    }
    
    /// Get display dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    
    /// Clear the back buffer to a specific color
    pub fn clear(&mut self, color: Rgb565) {
        let color_value = color.into_storage();
        for pixel in self.back_buffer.iter_mut() {
            *pixel = color_value;
        }
        self.dirty = true;
    }
    
    /// Set a pixel in the back buffer
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Rgb565) -> Result<(), DisplayError> {
        if x >= self.width || y >= self.height {
            return Err(DisplayError::InvalidCoordinate);
        }
        
        let index = (y as usize * self.width as usize) + x as usize;
        self.back_buffer[index] = color.into_storage();
        self.dirty = true;
        
        Ok(())
    }
    
    /// Get a pixel from the back buffer
    pub fn get_pixel(&self, x: u16, y: u16) -> Result<Rgb565, DisplayError> {
        if x >= self.width || y >= self.height {
            return Err(DisplayError::InvalidCoordinate);
        }
        
        let index = (y as usize * self.width as usize) + x as usize;
        Ok(Rgb565::new(
            ((self.back_buffer[index] >> 11) & 0x1F) as u8,
            ((self.back_buffer[index] >> 5) & 0x3F) as u8,
            (self.back_buffer[index] & 0x1F) as u8,
        ))
    }
    
    /// Fill a rectangle in the back buffer
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565) {
        let color_value = color.into_storage();
        
        // Clip to display bounds
        let end_x = (x + w).min(self.width);
        let end_y = (y + h).min(self.height);
        
        if end_x <= x || end_y <= y || x >= self.width || y >= self.height {
            return; // Nothing to draw
        }
        
        // Fill the rectangle in the back buffer
        for row in y..end_y {
            let start_index = (row as usize * self.width as usize) + x as usize;
            let end_index = (row as usize * self.width as usize) + end_x as usize;
            
            for pixel in &mut self.back_buffer[start_index..end_index] {
                *pixel = color_value;
            }
        }
        
        self.dirty = true;
    }
    
    /// Present the back buffer to the display (swap buffers)
    /// 
    /// This method:
    /// 1. Swaps the front and back buffers
    /// 2. Transfers the new front buffer to the physical display
    /// 3. Optimized to only transfer if there are changes
    pub fn present(&mut self) -> Result<(), DisplayError> {
        if !self.dirty {
            return Ok(()); // No changes to present
        }
        
        // Swap buffers
        core::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        
        // Transfer front buffer to display
        self.display.set_address_window(0, 0, self.width - 1, self.height - 1)
            .map_err(|_| DisplayError::SpiError)?;
        
        // Convert buffer to bytes and transfer in chunks for efficiency
        const CHUNK_SIZE: usize = 512; // Transfer 512 bytes (256 pixels) at a time
        let mut byte_buffer = [0u8; CHUNK_SIZE];
        
        for chunk in self.front_buffer.chunks(CHUNK_SIZE / 2) {
            let mut byte_index = 0;
            for &pixel in chunk {
                byte_buffer[byte_index] = (pixel >> 8) as u8;
                byte_buffer[byte_index + 1] = (pixel & 0xFF) as u8;
                byte_index += 2;
            }
            
            // Send this chunk to the display
            self.display.write_data(&byte_buffer[..byte_index])
                .map_err(|_| DisplayError::SpiError)?;
        }
        
        self.dirty = false;
        Ok(())
    }
    
    /// Force a full screen refresh (useful for initialization)
    pub fn force_refresh(&mut self) -> Result<(), DisplayError> {
        self.dirty = true;
        self.present()
    }
}

// Implement DrawTarget for embedded-graphics compatibility
impl<'a, SPI, DC> DrawTarget for DoubleBufferedDisplay<'a, SPI, DC>
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
            
            self.set_pixel(coord.x as u16, coord.y as u16, color)?;
        }
        Ok(())
    }
}

impl<'a, SPI, DC> OriginDimensions for DoubleBufferedDisplay<'a, SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
