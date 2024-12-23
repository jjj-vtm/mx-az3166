use core::{cell::RefCell, fmt::Write};

use cortex_m::interrupt::{self, Mutex};
use defmt::info;
// pick a panicking behavior
use defmt_rtt as _;

use panic_probe as _; // global logger

use ssd1306::{
    mode::DisplayConfig, prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface,
    Ssd1306,
};
use stm32f4xx_hal::{
    gpio::GpioExt,
    i2c::{I2c, Instance, Mode},
    pac::{self, I2C1, TIM5},
    rcc::{Clocks, RccExt},
    time::Hertz,
};

pub struct I2CProxy<'a, I2C: Instance> {
    pub i2c: &'a Mutex<RefCell<I2c<I2C>>>,
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

type DisplayType<I2C> = Ssd1306<
    ssd1306::prelude::I2CInterface<I2C>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;
type TempSensorType<I2C> = hts221::HTS221<I2C, stm32f4xx_hal::i2c::Error>;

pub struct Board<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub display: Option<DisplayType<I2C>>,
    pub temp_sensor: (Option<TempSensorType<I2C>>, Option<I2C>),
    pub shared_bus: Option<SharedBus>,
    pub clocks: Option<Clocks>,
    pub TIM5: Option<TIM5>,
}

type SharedBus = Mutex<RefCell<I2c<I2C1>>>;
type SharedBusT<'a> = I2CProxy<'a, I2C1>;

impl<'bus> Board<SharedBusT<'bus>> {

    /// Initializes the board peripahls and constructs the I2C bus
    pub fn construct_bus() -> Board<SharedBusT<'bus>> {
        let p = pac::Peripherals::take().unwrap();

        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpiob = p.GPIOB.split();
        // Configure I2C1
        let scl = gpiob.pb8;
        let sda = gpiob.pb9;

        let i2c = I2c::new(p.I2C1, (scl, sda), Mode::standard(Hertz::kHz(400)), &clocks);
        Board {
            display: None,
            temp_sensor: (None, None),
            clocks: Some(clocks),
            TIM5: Some(p.TIM5),
            shared_bus:  Some(Mutex::new(RefCell::new(i2c))),
        }
    }

    /// Finishes the construction using the bus
    /// /// # Examples
    ///
    /// ```
    ///    let bus = mxaz3166_board::Board::construct_bus();
    ///    let board = Board::initialize_periphals(&bus);
    ///    let mut display = board.display.unwrap();
    /// ```
    pub fn initialize_periphals(&'bus mut self) -> Self {
        let mut proxy1 = I2CProxy { i2c: self.shared_bus.as_ref().unwrap() };

        let proxy2 = I2CProxy { i2c: self.shared_bus.as_ref().unwrap() };

        let hts221 = hts221::Builder::new()
            .with_data_rate(hts221::DataRate::Continuous1Hz)
            .build(&mut proxy1)
            .unwrap();

        let interface = I2CDisplayInterface::new(proxy2);

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();

        Board {
            display: Some(display),
            temp_sensor: (Some(hts221), Some(proxy1)),
            shared_bus: None,
            clocks: self.clocks.take(),
            TIM5: self.TIM5.take(),
        }

       
    }
}
