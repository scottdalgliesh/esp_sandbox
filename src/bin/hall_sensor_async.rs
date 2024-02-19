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

use embedded_hal_async::digital::Wait;
use embassy_sync::channel::{Channel, Sender};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::{ClockControl, CpuClock},
    embassy,
    gpio::{AnyPin, Input, Output, PullUp, PushPull, IO},
    peripherals::Peripherals,
    prelude::*,
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
    mut sensor: AnyPin<Input<PullUp>>,
    sensor_id: u8,
    sender: Sender<'static, CriticalSectionRawMutex, SensorEvent, 1>,
) {
    loop {
        sensor.wait_for_any_edge().await.unwrap();
        Timer::after(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        if sensor.is_low().unwrap() {
            sender.send(SensorEvent::Closed(sensor_id)).await;
        } else {
            sender.send(SensorEvent::Released(sensor_id)).await;
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn output_manager(
    mut outputs: [AnyPin<Output<PushPull>>; 2],
    receiver: Receiver<'static, CriticalSectionRawMutex, SensorEvent, 1>,
) {
    loop {
        match receiver.receive().await {
            SensorEvent::Closed(n) => {
                outputs[n as usize].set_low().unwrap();
                println!("SENSOR {n}: CLOSED")
            }
            SensorEvent::Released(n) => {
                outputs[n as usize].set_high().unwrap();
                println!("SENSOR {n}: OPEN")
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    // Initialize hall sensor
    let hall_sensors = [
        io.pins.gpio8.into_pull_up_input().degrade(),
        io.pins.gpio20.into_pull_up_input().degrade(),
    ];

    // Initialize leds
    let mut leds = [
        io.pins.gpio2.into_push_pull_output().degrade(),
        io.pins.gpio3.into_push_pull_output().degrade(),
    ];
    for (i, (hall, led)) in hall_sensors.iter().zip(leds.iter_mut()).enumerate() {
        if hall.is_high().unwrap() {
            println!("Sensor {i} initial state: OPEN");
            led.set_high().unwrap();
        } else {
            println!("Sensor {i} initial state: CLOSED");
            led.set_low().unwrap();
        }
    }

    // Async requires the GPIO interrupt to wake futures
    hal::interrupt::enable(
        hal::peripherals::Interrupt::GPIO,
        hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let sender = CHANNEL.sender();
    let receiver = CHANNEL.receiver();

    // initialize async tasks
    for (i, hall) in hall_sensors.into_iter().enumerate() {
        spawner
            .spawn(sensor_watcher(hall, i as u8, sender.clone()))
            .unwrap();
    }
    spawner
        .spawn(output_manager(leds, receiver.clone()))
        .unwrap();

    println!("Tasks initialized")
}
