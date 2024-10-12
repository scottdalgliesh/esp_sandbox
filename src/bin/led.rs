//! Simple demo flashing an LED on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO0: LED

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
};
use esp_println::println;

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let delay = Delay::new();

    // Initialize led
    let mut led = Output::new(io.pins.gpio0, Level::High);

    // Event loop
    loop {
        led.toggle();
        let status = match led.is_set_low() {
            true => "LOW",
            false => "HIGH",
        };
        println!("LED {status}");
        delay.delay_millis(500);
    }
}
