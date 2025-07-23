//! Graphics and Drawing Utilities
//! 
//! Provides graphics primitives and drawing functions for TFT displays

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    Drawable,
};

/// Color palette for the display
pub mod colors {
    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    
    pub const BLACK: Rgb565 = Rgb565::BLACK;
    pub const WHITE: Rgb565 = Rgb565::WHITE;
    pub const RED: Rgb565 = Rgb565::RED;
    pub const GREEN: Rgb565 = Rgb565::GREEN;
    pub const BLUE: Rgb565 = Rgb565::BLUE;
    pub const YELLOW: Rgb565 = Rgb565::YELLOW;
    pub const MAGENTA: Rgb565 = Rgb565::MAGENTA;
    pub const CYAN: Rgb565 = Rgb565::CYAN;
}

/// Graphics helper functions
pub struct Graphics;

impl Graphics {
    /// Draw a filled rectangle
    pub fn draw_filled_rect<D>(
        display: &mut D,
        top_left: Point,
        size: Size,
        color: Rgb565,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Rectangle::new(top_left, size)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)
    }

    /// Draw a circle
    pub fn draw_circle<D>(
        display: &mut D,
        center: Point,
        radius: u32,
        color: Rgb565,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Circle::new(center - Point::new(radius as i32, radius as i32), radius * 2)
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
    }

    /// Draw text at a specific position
    pub fn draw_text<D>(
        _display: &mut D,
        _text: &str,
        _position: Point,
        _color: Rgb565,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Note: In a real implementation, you'd specify a font
        // For now, this is a placeholder that would need a font implementation
        Ok(())
    }
}
