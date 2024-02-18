// Simple test of hall sensor with indicator LED on ESP32C3
// Connections:
// hall sensor: GPIO 8
// LED: GPIO 2

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, gpio::IO, peripherals::Peripherals, prelude::*, Delay};

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    println!("Hello world!");

    // Initialize led
    let mut led = io.pins.gpio2.into_push_pull_output();
    led.set_low().unwrap();

    // Initialize hall sensor
    let hall = io.pins.gpio8.into_pull_up_input();

    // Event loop
    loop {
        let hall_status = match hall.is_low().unwrap() {
            true => {
                led.set_low().unwrap();
                "CLOSED"
            }
            false => {
                led.set_high().unwrap();
                "OPEN"
            }
        };
        println!("HALL SENSOR: {hall_status}");
        delay.delay_ms(500u32);
    }
}
