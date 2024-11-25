//! Simple demo flashing an LED on ESP32C3
//!
//! Connections List (TODO: wiring schematic)
//! - GPIO20: stepper (DRV8825 DIR)
//! - GPIO21: stepper (DRV8825 STEP)
//!
//! Example is written assuming the DRV8825 board is configured for 1/8 micro-steps

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output},
    timer::timg::TimerGroup,
};

const RPM: u32 = 60;
const NUM_STEPS: usize = 1600;
const PAUSE_SEC: u32 = 4;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize hardware
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Initialize led
    let mut _dir = Output::new(peripherals.GPIO20, Level::Low);
    let mut step = Output::new(peripherals.GPIO21, Level::Low);

    // calculate delay time for square wave
    let delay_time = (60 * 1_000_000 / RPM / NUM_STEPS as u32 / 2).into();
    log::info!("delay time (us): {delay_time}");

    // calculate cycle time (1 rotation + pause)
    let cycle_time = (60 * 1_000_000 / RPM + PAUSE_SEC * 1_000_000).into();
    log::info!("cycle time (us): {cycle_time}");

    // start cycle
    let overall_time = Instant::now();
    let mut cycle_ticker = Ticker::every(Duration::from_micros(cycle_time));

    // Event loop
    loop {
        // get time at the start of this rotation
        let initial_time = Instant::now();
        let mut ticker = Ticker::every(Duration::from_micros(delay_time));

        let mut high_start_times = [Instant::from_micros(0); NUM_STEPS];
        let mut low_start_times = [Instant::from_micros(0); NUM_STEPS];
        let mut end_cycle_times = [Instant::from_micros(0); NUM_STEPS];

        // perform 1 rotation
        for i in 0..NUM_STEPS {
            step.set_high();
            high_start_times[i] = Instant::now();
            ticker.next().await;

            step.set_low();
            low_start_times[i] = Instant::now();
            ticker.next().await;

            end_cycle_times[i] = Instant::now();
        }

        // log out timing data
        for i in 0..NUM_STEPS {
            let high_start_time = high_start_times[i];
            let low_start_time = low_start_times[i];
            let end_cycle_time = end_cycle_times[i];

            let high_time = low_start_time.duration_since(high_start_time).as_micros();
            let low_time = end_cycle_time.duration_since(low_start_time).as_micros();
            let total_cycle_time = end_cycle_time.duration_since(high_start_time).as_micros();
            let total_elapsed = end_cycle_time.duration_since(initial_time).as_millis();
            let overall_elapsed = end_cycle_time.duration_since(overall_time).as_secs();
            log::info!("step {i}: {high_time} - {low_time} - {total_cycle_time} - {total_elapsed} - {overall_elapsed}");
        }

        // wait until "pause" period ends
        cycle_ticker.next().await;
    }
}
