//! Simple demo ramping a dc motor via ESP32C3 & DBH12 driver
//!
//! Connections List (TODO: wiring schematic)
//! - GPIO6: motor A IN1 (DBH12 IN1)
//! - GPIO7: motor A IN2 (DBH12 IN2)
//! - GPIO9: button (momentary, wired to ground)
//! - GPIO21: motor A enable (DBH12 EN)
//!

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use {defmt_rtt as _, esp_backtrace as _};

// motor parameters
const PWM_MIN: u8 = 0;
const PWM_MAX: u8 = 95;
const RAMP_DURATION: u16 = 5000;
const PAUSE_DURATION: u64 = 5;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // disable driver and drive PWM pins low during setup
    let output_config = OutputConfig::default().with_pull(Pull::Down);
    let mut enable = Output::new(peripherals.GPIO21, Level::Low, output_config);
    let in0 = Output::new(peripherals.GPIO6, Level::Low, output_config);
    let in1 = Output::new(peripherals.GPIO7, Level::Low, output_config);

    // initialize embassy
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // initialize ledc peripheral
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // initialize ledc timers
    let timer_config = timer::config::Config {
        duty: timer::config::Duty::Duty8Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(1),
    };
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let mut lstimer1 = ledc.timer::<LowSpeed>(timer::Number::Timer1);
    lstimer0.configure(timer_config).unwrap();
    lstimer1.configure(timer_config).unwrap();

    // initialize pwm channels
    let mut channel0 = ledc.channel(channel::Number::Channel0, in0);
    let mut channel1 = ledc.channel(channel::Number::Channel1, in1);
    configure_channel(&mut channel0, &lstimer0);
    configure_channel(&mut channel1, &lstimer1);

    // initialize input button
    let mut input = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    loop {
        // wait for input button to be triggered
        info!("waiting for input...");
        input.wait_for_falling_edge().await;

        // enable driver
        enable.set_high();

        // motor demo
        info!("starting forward ramp up");
        channel0
            .start_duty_fade(PWM_MIN, PWM_MAX, RAMP_DURATION)
            .unwrap();
        while channel0.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting forward ramp down");
        channel0
            .start_duty_fade(PWM_MAX, PWM_MIN, RAMP_DURATION)
            .unwrap();
        while channel0.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting backward ramp up");
        channel1
            .start_duty_fade(PWM_MIN, PWM_MAX, RAMP_DURATION)
            .unwrap();
        while channel1.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting backward ramp down");
        channel1
            .start_duty_fade(PWM_MAX, PWM_MIN, RAMP_DURATION)
            .unwrap();
        while channel1.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        // disable driver
        enable.set_low();

        info!("pausing");
        Timer::after_secs(PAUSE_DURATION).await;
    }
}

/// Configure ledc channel for PWM output.
fn configure_channel<'a>(
    channel: &mut channel::Channel<'a, LowSpeed>,
    timer: &'a timer::Timer<'a, LowSpeed>,
) {
    let config = channel::config::Config {
        timer,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    };
    channel.configure(config).unwrap()
}
