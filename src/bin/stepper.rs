//! Simple demo periodically rotating a stepper motor via ESP32C3 & DRV8825
//!
//! Connections List (TODO: wiring schematic)
//! - GPIO20: stepper (DRV8825 DIR)
//! - GPIO21: stepper (DRV8825 STEP)
//!
//! Example is written assuming the DRV8825 board is configured for 1/8 micro-steps

#![no_std]
#![no_main]

use defmt::info;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
};
use {defmt_rtt as _, esp_backtrace as _};

const RPM: u32 = 60;
const NUM_STEPS: u32 = 1600;

#[main]
fn main() -> ! {
    // Initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // Initialize led
    let output_config = OutputConfig::default();
    let mut _dir = Output::new(peripherals.GPIO20, Level::High, output_config);
    let mut step = Output::new(peripherals.GPIO21, Level::Low, output_config);

    // Calculate delay time for square wave
    let delay_time: u32 = 60 * 1_000_000 / RPM / NUM_STEPS / 2;
    info!("delay time: {}", delay_time);

    // Event loop
    let mut counter = 0;
    loop {
        info!("{}: start rotation", counter);
        for _ in 0..NUM_STEPS {
            step.set_high();
            delay.delay_micros(delay_time);
            step.set_low();
            delay.delay_micros(delay_time);
        }
        info!("pause");
        delay.delay_millis(2_000);
        counter += 1;
    }
}
