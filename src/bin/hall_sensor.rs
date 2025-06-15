//! Simple test of hall sensor with indicator LED on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO 2: LED
//! - GPIO 8: hall effect sensor

#![no_std]
#![no_main]

use defmt::info;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
};
use {defmt_rtt as _, esp_backtrace as _};

#[main]
fn main() -> ! {
    // Initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // Initialize led & hall sensor
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let hall = Input::new(
        peripherals.GPIO8,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Event loop
    loop {
        let hall_level = hall.level();
        let status = if hall_level.into() { "OPEN" } else { "CLOSED" };
        led.set_level(hall_level);
        info!("HALL SENSOR: {}", status);
        delay.delay_millis(500);
    }
}
