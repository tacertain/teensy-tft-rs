#![no_std]

//! Teensy TFT Display Library
//! 
//! This library provides functionality for controlling TFT displays 
//! on Teensy microcontrollers using embedded Rust.

pub mod display;
pub mod graphics;
pub mod dma_display;

pub use display::{TftDisplay, DoubleBufferedDisplay, DisplayError};
pub use dma_display::{DmaTftDisplay, DmaDoubleBufferedDisplay};
pub use graphics::*;
