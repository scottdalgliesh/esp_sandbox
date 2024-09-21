//! Simple demo of light sleep functionality of ESP32C3
//!
//! Connections List (see schematic for 'led.rs' for details)
//! - GPIO0: LED

#![no_std]
#![no_main]

use core::time::Duration;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    rtc_cntl::{sleep::TimerWakeupSource, Rtc},
    Delay,
};
use esp_println::println;

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    let mut rtc = Rtc::new(peripherals.LPWR);

    // Initialize led
    println!("start");
    let mut led = io.pins.gpio21.into_push_pull_output();
    led.set_high().unwrap();
    delay.delay_ms(5000u32);
    led.set_low().unwrap();

    // enter light sleep
    println!("sleep");
    let timer = TimerWakeupSource::new(Duration::from_secs(5));
    delay.delay_ms(100u32);
    rtc.sleep_light(&[&timer], &mut delay);

    // note that logging via println!() stops working at this point, as communication
    // with host computer is lost during light sleep. Connection will not be resumed
    // after wake-up, so println! is not used below

    // blink LED to show process is resumed after wake-up
    loop {
        led.toggle().unwrap();
        delay.delay_ms(1000u32);
    }

    // note regarding deep sleep: deep sleep will shut down nearly all MCU processes
    // meaning that upon wake-up, program will not resume in-place, but restart the
    // entire program from the top.
}
