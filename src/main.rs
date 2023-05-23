#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, gpio::IO, peripherals::Peripherals, prelude::*, Delay, Rtc};

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    // Disable the RTC watchdog timer
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();
    println!("Hello world!");

    // Initialize led
    let mut led = io.pins.gpio2.into_push_pull_output();
    led.set_high().unwrap();

    // Event loop
    loop {
        led.toggle().unwrap();
        let status = match led.is_set_low().unwrap() {
            true => "LOW",
            false => "HIGH",
        };
        println!("LED {status}");
        delay.delay_ms(500u32);
    }
}
