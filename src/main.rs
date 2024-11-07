#![no_std]
#![no_main]
#![macro_use]

use core::{cell::RefCell, fmt::Write};

use cortex_m::interrupt::{self, Mutex};
use defmt::info;
// pick a panicking behavior
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use heapless::String;
use panic_probe as _; // global logger

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
    i2c::{I2c, Mode},
    pac::{self},
    rcc::RccExt,
    time::Hertz,
    timer::TimerExt,
};

struct I2CProxy<'a> {
    i2c: &'a Mutex<RefCell<I2c<pac::I2C1>>>,
}
#[derive(Debug)]
pub struct ErrorT {}

impl embedded_hal::i2c::Error for ErrorT {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

impl embedded_hal::i2c::ErrorType for I2CProxy<'_> {
    type Error = ErrorT;
}

impl embedded_hal::i2c::I2c for I2CProxy<'_> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        interrupt::free(|cs| {
            let bus = self.i2c.borrow(cs);
            let mut bus_m = bus.borrow_mut();

            let _ = bus_m.transaction_slice(address, operations);
        });
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    info!("Starting up");

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let mut delay = p.TIM5.delay_us(&clocks);

    let gpiob = p.GPIOB.split();
    // Configure I2C1
    let scl = gpiob.pb8;
    let sda = gpiob.pb9;

    let i2c = I2c::new(p.I2C1, (scl, sda), Mode::standard(Hertz::kHz(400)), &clocks);
    let i2c_m = Mutex::new(RefCell::new(i2c));

    let mut proxy1 = I2CProxy { i2c: &i2c_m };

    let proxy2 = I2CProxy { i2c: &i2c_m };

    let mut hts221 = hts221::Builder::new()
        .with_data_rate(hts221::DataRate::Continuous1Hz)
        .build(&mut proxy1)
        .unwrap();

    let mut buffer: String<32> = String::new();

    let interface = I2CDisplayInterface::new(proxy2);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let deg_c = hts221.temperature_x8(&mut proxy1).unwrap() as f32 / 8.0;

        info!("Temp: {:?}", deg_c);
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
