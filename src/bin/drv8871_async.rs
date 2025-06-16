//! Simple demo ramping a dc motor via ESP32C3 & DRV8871
//!
//! Connections List (TODO: wiring schematic)
//! - GPIO8: motor 1 (DRV8871 IN1)
//! - GPIO9: motor 2 (DRV8871 IN2)
//!

#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use {defmt_rtt as _, esp_backtrace as _};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // initialize hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
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

    //initialize pwm channels
    let mut channel0 = ledc.channel(channel::Number::Channel0, peripherals.GPIO8);
    let mut channel1 = ledc.channel(channel::Number::Channel1, peripherals.GPIO9);
    configure_channel(&mut channel0, &lstimer0);
    configure_channel(&mut channel1, &lstimer1);

    loop {
        info!("starting forward ramp up");
        channel0.start_duty_fade(0, 100, 2500).unwrap();
        while channel0.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting forward ramp down");
        channel0.start_duty_fade(100, 0, 2500).unwrap();
        while channel0.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting backward ramp up");
        channel1.start_duty_fade(0, 100, 2500).unwrap();
        while channel1.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("starting backward ramp down");
        channel1.start_duty_fade(100, 0, 2500).unwrap();
        while channel1.is_duty_fade_running() {
            Timer::after_millis(10).await;
        }

        info!("pausing");
        Timer::after_secs(5).await;
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
