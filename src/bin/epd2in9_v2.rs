//! Demo of Waveshare EPaper Display (2.9" v2), loosely based on example code
//! from the [epd-waveshare repo](https://github.com/caemor/epd-waveshare)
//!
//! Connections List (see schematic for details)
//! - GPIO 2: RST
//! - GPIO 3: BUSY
//! - GPIO 8: SCLK
//! - GPIO 9: DC
//! - GPIO 10: MOSI (DIN)
//! - GPIO 20: CS
//! SPI connections above correspond to pinout of seeed studio xiao esp32c3

#![no_std]
#![no_main]

// use embedded_hal::spi::SpiDevice;
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{
    epd2in9_v2::{Display2in9, Epd2in9},
    graphics::DisplayRotation,
    prelude::*,
};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    Delay,
};
use esp_println::println;

fn draw_text(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

fn draw_rotated_text(display: &mut Display2in9, text: &str, rotation: DisplayRotation) {
    display.set_rotation(rotation);
    draw_text(display, text, 5, 50);
}

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    // Identify pins
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sck = io.pins.gpio8.into_push_pull_output();
    let mosi = io.pins.gpio10.into_push_pull_output();
    let cs = io.pins.gpio20.into_push_pull_output();
    let dc = io.pins.gpio9.into_push_pull_output();
    let busy = io.pins.gpio3.into_floating_input();
    let rst = io.pins.gpio2.into_push_pull_output();

    // Initialize SPI & EPD
    println!("Initializing display");
    let spi = Spi::new(peripherals.SPI2, 8u32.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sck)
        .with_mosi(mosi);
    let mut spi_device = ExclusiveDevice::new(spi, cs, delay);
    let mut epd = Epd2in9::new(&mut spi_device, busy, dc, rst, &mut delay, Some(0)).unwrap();
    let mut display = Display2in9::default();

    // Text output and rotation demo
    println!("Begin text output and rotation demo");
    draw_rotated_text(&mut display, "Rotate 0!", DisplayRotation::Rotate0);
    draw_rotated_text(&mut display, "Rotate 90!", DisplayRotation::Rotate90);
    draw_rotated_text(&mut display, "Rotate 180!", DisplayRotation::Rotate180);
    draw_rotated_text(&mut display, "Rotate 270!", DisplayRotation::Rotate270);
    epd.update_frame(&mut spi_device, display.buffer(), &mut delay)
        .unwrap();
    epd.display_frame(&mut spi_device, &mut delay).unwrap();
    delay.delay_ms(1_000 as u32);

    // Clock graphic demo
    println!("Begin clock graphics demo");
    display.clear(Color::White).ok();
    let thin = PrimitiveStyle::with_stroke(Color::Black, 1);
    let thick = PrimitiveStyle::with_stroke(Color::Black, 4);
    let _ = Circle::with_center(Point::new(64, 64), 80)
        .into_styled(thin)
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(thick)
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 40))
        .into_styled(thin)
        .draw(&mut display);
    epd.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)
        .unwrap();
    delay.delay_ms(1_000 as u32);

    // Partial refresh demo - moving message
    println!("Begin partial quick refresh demo - moving message");
    epd.set_refresh(&mut spi_device, &mut delay, RefreshLut::Quick)
        .unwrap();
    display.clear(Color::White).ok();
    for i in 0..10 {
        draw_text(&mut display, "  Hello World! ", 5 + i * 12, 50);
        epd.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)
            .unwrap();
    }
    delay.delay_ms(1_000 as u32);

    // Partial refresh demo - spinner
    println!("Begin spinner demo");
    let spinner = ["|", "/", "-", "\\"];
    for i in 0..10 {
        display.clear(Color::White).ok();
        draw_text(&mut display, spinner[i % spinner.len()], 10, 100);
        epd.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)
            .unwrap();
    }
    delay.delay_ms(1_000 as u32);

    // display complete message and enter sleep
    println!("Complete");
    display.clear(Color::White).unwrap();
    draw_text(&mut display, "COMPLETE", 100, 60);
    epd.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)
        .unwrap();
    epd.sleep(&mut spi_device, &mut delay).unwrap();
    loop {}
}
