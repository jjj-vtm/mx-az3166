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
use stm32f4xx_hal::{gpio::GpioExt, i2c::{I2c, Mode}, pac, rcc::RccExt, time::{Hertz, KiloHertz}};

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
    
    loop {
        // your code goes here
    }
}
