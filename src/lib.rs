#![no_std]

//! Teensy TFT Display Library
//! 
//! This library provides functionality for controlling TFT displays 
//! on Teensy microcontrollers using embedded Rust.

pub mod display;
pub mod graphics;

pub use display::*;
pub use graphics::*;
