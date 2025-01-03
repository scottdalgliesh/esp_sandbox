//! Simple demo flashing an LED on ESP32C3
//!
//! Connections List (TODO: wiring schematic)
//! - GPIO20: stepper (DRV8825 DIR)
//! - GPIO21: stepper (DRV8825 STEP)
//!
//! Example is written assuming the DRV8825 board is configured per MICRO_STEP_MODE_DIVISOR value
//! e.g. MICRO_STEP_MODE_DIVISOR = 8 -> 1/8th step mode

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker};
use esp_backtrace as _;
use esp_hal::{
    gpio::{AnyPin, Level, Output},
    interrupt::{software::SoftwareInterruptControl, Priority},
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_hal_embassy::InterruptExecutor;
use static_cell::StaticCell;

// Inputs
const MOTOR_STEPS_PER_REV: u32 = 200;
const MICRO_STEP_MODE_DIVISOR: u32 = 2;
const RPM: u32 = 240;
const NUM_REVS: u32 = 10;
const PAUSE_SEC: u32 = 10;

// Calculated values
const NUM_STEPS: u32 = NUM_REVS * MOTOR_STEPS_PER_REV * MICRO_STEP_MODE_DIVISOR;
const DELAY_TIME_US: u32 =
    60 * 1_000_000 / (2 * RPM * MOTOR_STEPS_PER_REV * MICRO_STEP_MODE_DIVISOR);
const STEP_TIME_US: u32 = DELAY_TIME_US * 2;
const RUN_TIME_US: u32 = NUM_STEPS * STEP_TIME_US;
const TOTAL_CYCLE_TIME_US: u32 = RUN_TIME_US + PAUSE_SEC * 1_000_000;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize hardware
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    // Initialize embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let timer0: AnyTimer = timg0.timer0.into();
    let timer1: AnyTimer = timg1.timer0.into();
    esp_hal_embassy::init([timer0, timer1]);

    static EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();
    let executor = InterruptExecutor::new(sw_ints.software_interrupt2);
    let executor = EXECUTOR.init(executor);
    let spawner = executor.start(Priority::Priority3);
    spawner.must_spawn(pwm_manager(
        peripherals.GPIO20.into(),
        peripherals.GPIO21.into(),
    ));
}

#[embassy_executor::task]
async fn pwm_manager(dir_pin: AnyPin, step_pin: AnyPin) {
    log::info!("delay time (us): {DELAY_TIME_US}");
    log::info!("cycle time (us): {RUN_TIME_US}");

    // Initialize motor control GPIO
    let mut _dir = Output::new(dir_pin, Level::High);
    let mut step = Output::new(step_pin, Level::Low);

    // start cycle
    let overall_time = Instant::now();
    let mut cycle_ticker = Ticker::every(Duration::from_micros(TOTAL_CYCLE_TIME_US.into()));

    // for logging key times
    let mut high_start_times = [Instant::from_micros(0); NUM_STEPS as usize];
    let mut low_start_times = [Instant::from_micros(0); NUM_STEPS as usize];
    let mut end_cycle_times = [Instant::from_micros(0); NUM_STEPS as usize];

    // Event loop
    loop {
        // get time at the start of this rotation
        let initial_time = Instant::now();
        let mut ticker = Ticker::every(Duration::from_micros(DELAY_TIME_US.into()));

        // perform 1 rotation
        for i in 0..NUM_STEPS as usize {
            step.set_high();
            high_start_times[i] = Instant::now();
            ticker.next().await;

            step.set_low();
            low_start_times[i] = Instant::now();
            ticker.next().await;

            end_cycle_times[i] = Instant::now();
        }

        // log out timing data
        for i in 0..NUM_STEPS as usize {
            if i % 100 != 0 {
                continue;
            }
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
