//! Async test of multiple hall sensors with indicator LEDs on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO 2: LED 1
//! - GPIO 3: LED 2
//! - GPIO 8: hall effect sensor 1
//! - GPIO 20: hall effect sensor 2

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    timer::timg::TimerGroup,
};
use {defmt_rtt as _, esp_backtrace as _};

const DEBOUNCE_DELAY_MS: u64 = 1;

/// Indicate current status of sensor via LED
fn show_sensor_status(id: u8, sensor: &mut Input, led: &mut Output) {
    // report change
    let level = sensor.level();
    let status = if level.into() { "OPEN" } else { "CLOSED" };
    led.set_level(level);
    info!("SENSOR {}: {}", id, status);
}

/// Monitor sensor and indicate status via LED
#[embassy_executor::task(pool_size = 2)]
async fn sensor_watcher(id: u8, mut sensor: Input<'static>, mut led: Output<'static>) {
    loop {
        sensor.wait_for_any_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        show_sensor_status(id, &mut sensor, &mut led);
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Initialize hall sensors
    let input_config = InputConfig::default().with_pull(Pull::Up);
    let mut hall_sensors = [
        Input::new(peripherals.GPIO8, input_config),
        Input::new(peripherals.GPIO20, input_config),
    ];

    // Initialize leds
    let output_config = OutputConfig::default();
    let mut leds = [
        Output::new(peripherals.GPIO2, Level::Low, output_config),
        Output::new(peripherals.GPIO3, Level::Low, output_config),
    ];

    // Set LED based on initial state of hall sensor
    for (i, (hall, led)) in hall_sensors.iter_mut().zip(leds.iter_mut()).enumerate() {
        show_sensor_status(i as u8, hall, led);
    }

    // Initialize async tasks
    for (i, (hall, led)) in hall_sensors.into_iter().zip(leds.into_iter()).enumerate() {
        spawner.spawn(sensor_watcher(i as u8, hall, led)).unwrap();
    }

    info!("Monitoring sensors...")
}
