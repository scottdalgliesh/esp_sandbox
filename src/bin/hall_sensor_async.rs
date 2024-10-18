//! Async test of multiple hall sensors with indicator LEDs on ESP32C3
//!
//! Connections List (see schematic for details)
//! - GPIO 2: LED 1
//! - GPIO 3: LED 2
//! - GPIO 8: hall effect sensor 1
//! - GPIO 20: hall effect sensor 2

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_sync::channel::{Channel, Sender};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Io, Level, Output, Pull},
    timer::timg::TimerGroup,
};

static CHANNEL: Channel<CriticalSectionRawMutex, SensorEvent, 1> = Channel::new();
const DEBOUNCE_DELAY_MS: u64 = 1;

#[derive(Clone, Copy)]
enum SensorEvent {
    Closed(u8),
    Released(u8),
}

#[embassy_executor::task(pool_size = 2)]
async fn sensor_watcher(
    mut sensor: Input<'static>,
    sensor_id: u8,
    sender: Sender<'static, CriticalSectionRawMutex, SensorEvent, 1>,
) {
    loop {
        sensor.wait_for_any_edge().await;
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        if sensor.is_low() {
            sender.send(SensorEvent::Closed(sensor_id)).await;
        } else {
            sender.send(SensorEvent::Released(sensor_id)).await;
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn output_manager(
    mut outputs: [Output<'static>; 2],
    receiver: Receiver<'static, CriticalSectionRawMutex, SensorEvent, 1>,
) {
    loop {
        match receiver.receive().await {
            SensorEvent::Closed(n) => {
                outputs[n as usize].set_low();
                log::info!("SENSOR {n}: CLOSED")
            }
            SensorEvent::Released(n) => {
                outputs[n as usize].set_high();
                log::info!("SENSOR {n}: OPEN")
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize hardware
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // initialize embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Initialize hall sensor
    let hall_sensors = [
        Input::new(io.pins.gpio8, Pull::Up),
        Input::new(io.pins.gpio20, Pull::Up),
    ];

    // Initialize leds
    let mut leds = [
        Output::new(io.pins.gpio2, Level::Low),
        Output::new(io.pins.gpio3, Level::Low),
    ];

    for (i, (hall, led)) in hall_sensors.iter().zip(leds.iter_mut()).enumerate() {
        if hall.is_high() {
            log::info!("Sensor {i} initial state: OPEN");
            led.set_high();
        } else {
            log::info!("Sensor {i} initial state: CLOSED");
            led.set_low();
        }
    }

    let sender = CHANNEL.sender();
    let receiver = CHANNEL.receiver();

    // initialize async tasks
    for (i, hall) in hall_sensors.into_iter().enumerate() {
        spawner
            .spawn(sensor_watcher(hall, i as u8, sender))
            .unwrap();
    }
    spawner.spawn(output_manager(leds, receiver)).unwrap();

    log::info!("Tasks initialized")
}
