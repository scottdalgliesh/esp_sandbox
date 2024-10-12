//! Simple demo of light sleep functionality of ESP32C3
//!
//! Connections List (see schematic for 'led.rs' for details)
//! - GPIO0: LED

#![no_std]
#![no_main]

use core::time::Duration;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
};
use esp_println::println;

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let delay = Delay::new();
    let mut rtc = Rtc::new(peripherals.LPWR);

    // Initialize led
    println!("start");
    let mut led = Output::new(io.pins.gpio0, Level::High);
    delay.delay_millis(5000);
    led.set_low();

    // enter light sleep
    println!("sleep");
    let timer = TimerWakeupSource::new(Duration::from_secs(5));
    delay.delay_millis(100);
    rtc.sleep_light(&[&timer]);

    // note that logging via println!() stops working at this point, as communication
    // with host computer is lost during light sleep. Connection will not be resumed
    // after wake-up, so println! is not used below

    // blink LED to show process is resumed after wake-up
    loop {
        led.toggle();
        delay.delay_millis(1000);
    }

    // note regarding deep sleep: deep sleep will shut down nearly all MCU processes
    // meaning that upon wake-up, program will not resume in-place, but restart the
    // entire program from the top.
}
