//! Simple test of hall sensor with indicator LED on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO 2: LED
//! - GPIO 8: hall effect sensor

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pull},
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
    let mut led = Output::new(io.pins.gpio2, Level::Low);

    // Initialize hall sensor
    let hall = Input::new(io.pins.gpio8, Pull::Up);

    // Event loop
    loop {
        let hall_status = match hall.is_low() {
            true => {
                led.set_low();
                "CLOSED"
            }
            false => {
                led.set_high();
                "OPEN"
            }
        };
        println!("HALL SENSOR: {hall_status}");
        delay.delay_millis(500);
    }
}
