// Async test of hall sensor with indicator LED on ESP32C3
// Connections:
// hall sensor: GPIO 8
// LED: GPIO 2

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_sync::channel::{Channel, Sender};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::{ClockControl, CpuClock},
    embassy,
    gpio::{Gpio2, Gpio8, Input, Output, PullUp, PushPull, IO},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc,
};

static CHANNEL: Channel<CriticalSectionRawMutex, SensorEvent, 1> = Channel::new();
const DEBOUNCE_DELAY_MS: u64 = 1;

#[derive(Clone, Copy)]
enum SensorEvent {
    Closed(u8),
    Released(u8),
}

#[embassy_executor::task(pool_size = 1)]
async fn sensor_watcher(
    mut sensor: Gpio8<Input<PullUp>>,
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
    mut output: Gpio2<Output<PushPull>>,
    receiver: Receiver<'static, CriticalSectionRawMutex, SensorEvent, 1>,
) {
    loop {
        match receiver.recv().await {
            SensorEvent::Closed(1) => {
                output.set_low().unwrap();
                println!("CLOSED")
            }
            SensorEvent::Released(1) => {
                output.set_high().unwrap();
                println!("OPEN")
            }
            _ => (),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    // disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    // Initialize hall sensor
    let hall = io.pins.gpio8.into_pull_up_input();

    // Initialize led
    let mut led = io.pins.gpio2.into_push_pull_output();
    if hall.is_high().unwrap() {
        println!("Initial state: OPEN");
        led.set_high().unwrap();
    } else {
        println!("Initial state: CLOSED");
        led.set_low().unwrap();
    }

    // Async requires the GPIO interrupt to wake futures
    hal::interrupt::enable(
        hal::peripherals::Interrupt::GPIO,
        hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let sender = CHANNEL.sender();
    let receiver = CHANNEL.receiver();

    spawner
        .spawn(sensor_watcher(hall, 1, sender.clone()))
        .unwrap();
    spawner
        .spawn(output_manager(led, receiver.clone()))
        .unwrap();
    println!("Tasks initialized")
}
