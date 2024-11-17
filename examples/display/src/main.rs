#![no_std]
#![no_main]
#![macro_use]

use core::{cell::RefCell, fmt::Write};

use cortex_m::{
    delay,
    interrupt::{self, Mutex},
};
use defmt::info;
use mxaz3166_board::*;
// pick a panicking behavior
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use heapless::String;
use panic_probe as _; // global logger

use mxaz3166_board::*;

use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::mode::DisplayConfig;
use stm32f4xx_hal::{
    i2c::{I2c, Instance, Mode},
    pac::{self},
    timer::TimerExt,
};

#[entry]
fn main() -> ! {
    let mut board = mxaz3166_board::Board::construct_bus();
    
    let mut board = Board::initialize_periphals(&mut board);

    let mut delay = board.TIM5.take().unwrap().delay_us(&board.clocks.unwrap());

    let mut display = board.display.unwrap();
    let mut hts221 = board.temp_sensor.0.unwrap();
    let mut hts_bus = board.temp_sensor.1.unwrap();

    info!("Starting up");

    let mut buffer: String<32> = String::new();

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Test", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    loop {
        let deg_c = hts221.temperature_x8(&mut hts_bus).unwrap() as f32 / 8.0;

        write!(buffer, "Temperatur: {:?}", deg_c).unwrap();

        Text::with_baseline(buffer.as_str(), Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();

        delay.delay_ms(1000);
        buffer.clear();
        display.clear_buffer();
    }
}
