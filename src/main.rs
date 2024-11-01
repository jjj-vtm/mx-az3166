#![no_std]
#![no_main]
#![macro_use]

use defmt::info;
// pick a panicking behavior
use defmt_rtt as _;
use panic_probe as _; // global logger

use cortex_m::asm;
use cortex_m_rt::entry;
use ssd1306::{mode::DisplayConfig, prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306};
use stm32f4xx_hal::{gpio::GpioExt, i2c::{I2c, Mode}, pac, rcc::RccExt, time::Hertz};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};


#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    let p = pac::Peripherals::take().unwrap();
    info!("Starting up");

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    
    let gpiob = p.GPIOB.split();
    // Configure I2C1
    let scl = gpiob.pb8;
    let sda = gpiob.pb9;

    let i2c = I2c::new(p.I2C1, (scl, sda), Mode::standard(Hertz::kHz(400)), &clocks);    


    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();
    
    loop {
        // your code goes here
    }
}
