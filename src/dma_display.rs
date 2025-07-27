//! DMA-enabled TFT Display Driver Module
//! 
//! Advanced display interface with DMA support for high-performance graphics

use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayMs;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb565,
};
use futures::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::display::{DisplayError, Delay};

/// DMA-enabled TFT display driver
pub struct DmaTftDisplay<SPI, DC> {
    spi: SPI,
    dc: DC,
    width: u16,
    height: u16,
}

impl<SPI, DC> DmaTftDisplay<SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    /// Create a new DMA-enabled TFT display instance
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self {
            spi,
            dc,
            width: 240,
            height: 320,
        }
    }

    /// Initialize the display (same as regular TftDisplay)
    pub fn init(&mut self) -> Result<(), DisplayError> {
        use embedded_hal::blocking::delay::DelayMs;
        
        let mut delay = Delay;
        
        // Reset pin is connected to 3V externally, so no reset sequence needed
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

    /// Write data to the display (blocking)
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
}

// For non-DMA SPI types, provide async interface compatibility
impl<SPI, DC> DmaTftDisplay<SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    /// Async DMA write data to display (fallback to blocking for non-DMA SPI)
    pub fn write_data_dma<'a>(&'a mut self, data: &'a [u8]) -> impl Future<Output = Result<DmaResult, DisplayError>> + 'a {
        DmaWriteFuture {
            display: self,
            data,
            completed: false,
        }
    }
}

/// Result type that includes whether DMA was actually used
#[derive(Debug)]
pub struct DmaResult {
    pub success: bool,
    pub used_dma: bool,
}

impl DmaResult {
    pub fn new(success: bool, used_dma: bool) -> Self {
        Self { success, used_dma }
    }
}

/// Future for DMA write operations (fallback implementation)
struct DmaWriteFuture<'a, SPI, DC> {
    display: &'a mut DmaTftDisplay<SPI, DC>,
    data: &'a [u8],
    completed: bool,
}

impl<'a, SPI, DC> Future for DmaWriteFuture<'a, SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    type Output = Result<DmaResult, DisplayError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        
        if this.completed {
            return Poll::Ready(Ok(DmaResult::new(true, false))); // Success, but no DMA
        }

        // For non-DMA SPI, fall back to blocking write
        match this.display.write_data(this.data) {
            Ok(()) => {
                this.completed = true;
                // Return success=true, used_dma=false since we're using fallback
                Poll::Ready(Ok(DmaResult::new(true, false)))
            }
            Err(_) => Poll::Ready(Err(DisplayError::SpiError)),
        }
    }
}

/// DMA-enabled double-buffered display for high-performance graphics
pub struct DmaDoubleBufferedDisplay<'a, SPI, DC> {
    display: DmaTftDisplay<SPI, DC>,
    // Frame buffers provided externally - RGB565 format (2 bytes per pixel)
    back_buffer: &'a mut [u16],
    front_buffer: &'a mut [u16],
    width: u16,
    height: u16,
    dirty: bool, // Track if back buffer has changes
}

impl<'a, SPI, DC> DmaDoubleBufferedDisplay<'a, SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    /// Create a new DMA double-buffered display with externally provided buffers
    pub fn new(
        mut display: DmaTftDisplay<SPI, DC>,
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
        
        Ok(Self {
            display,
            back_buffer,
            front_buffer,
            width,
            height,
            dirty: false,
        })
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
    
    /// Present the back buffer to the display using DMA (async)
    pub async fn present_async(&mut self) -> Result<DmaResult, DisplayError> {
        if !self.dirty {
            // No changes to present, but consider this as not using DMA
            return Ok(DmaResult::new(true, false));
        }
        
        // Swap buffers
        core::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        
        // Set address window for full screen update
        self.display.set_address_window(0, 0, self.width - 1, self.height - 1)
            .map_err(|_| DisplayError::SpiError)?;
        
        // Convert buffer to bytes for DMA transfer
        const CHUNK_SIZE: usize = 1024; // Larger chunks for DMA efficiency
        let mut byte_buffer = [0u8; CHUNK_SIZE];
        let mut any_dma_used = false;
        
        for chunk in self.front_buffer.chunks(CHUNK_SIZE / 2) {
            let mut byte_index = 0;
            for &pixel in chunk {
                byte_buffer[byte_index] = (pixel >> 8) as u8;
                byte_buffer[byte_index + 1] = (pixel & 0xFF) as u8;
                byte_index += 2;
            }
            
            // Send this chunk to the display using DMA
            let result = self.display.write_data_dma(&byte_buffer[..byte_index]).await?;
            if result.used_dma {
                any_dma_used = true;
            }
        }
        
        self.dirty = false;
        Ok(DmaResult::new(true, any_dma_used))
    }
    
    /// Present the back buffer to the display (blocking fallback)
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
impl<'a, SPI, DC> DrawTarget for DmaDoubleBufferedDisplay<'a, SPI, DC>
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

impl<'a, SPI, DC> OriginDimensions for DmaDoubleBufferedDisplay<'a, SPI, DC>
where
    SPI: Write<u8>,
    DC: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}
