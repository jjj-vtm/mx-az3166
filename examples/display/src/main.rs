#![no_std]
#![no_main]
#![macro_use]

use core::{cell::RefCell, fmt::Write};

use cortex_m::interrupt::{self, Mutex};
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
use ssd1306::{
    mode::DisplayConfig, prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface,
    Ssd1306,
};
use stm32f4xx_hal::{
    gpio::GpioExt,
    i2c::{I2c, Instance, Mode},
    pac::{self},
    rcc::RccExt,
    time::Hertz,
    timer::TimerExt,
};

struct I2CProxy<'a, I2C: Instance> {
    i2c: &'a Mutex<RefCell<I2c<I2C>>>,
}

impl<I2C> embedded_hal::i2c::ErrorType for I2CProxy<'_, I2C>
where
    I2C: Instance,
{
    type Error = stm32f4xx_hal::i2c::Error;
}

impl<I2C> embedded_hal::i2c::I2c for I2CProxy<'_, I2C>
where
    I2C: Instance,
{
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        interrupt::free(|cs| {
            let mut bus = self.i2c.borrow(cs).borrow_mut();
            bus.transaction_slice(address, operations)
        })
    }
}

#[entry]
fn main() -> ! {
    let bus = mxaz3166_board::Board::construct_bus();
    let board = Board::initialize_periphals(&bus);

    let mut display = board.display.unwrap();
    let hts221 = board.temp_sensor.unwrap();
    info!("Starting up");

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Test", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();


    display.clear_buffer();

    loop {
        // let deg_c = hts221.temperature_x8(&mut proxy1).unwrap() as f32 / 8.0;

        // info!("Temp: {:?}", deg_c);
        //write!(buffer, "Temperatur: {:?}", deg_c).unwrap();

            // delay.delay_ms(1000);
    // buffer.clear();
    }
}
