//! Simple demo flashing an LED on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO0: LED

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output},
    prelude::*,
};

#[entry]
fn main() -> ! {
    // Initialize hardware
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // Initialize led
    let mut led = Output::new(peripherals.GPIO0, Level::High);

    // Event loop
    loop {
        led.toggle();
        let status = if led.is_set_high() { "ON" } else { "OFF" };
        log::info!("LED {status}");
        delay.delay_millis(500);
    }
}
