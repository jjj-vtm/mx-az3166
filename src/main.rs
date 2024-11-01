#![no_std]
#![no_main]
#![macro_use]

// pick a panicking behavior
use panic_probe as _;
use defmt_rtt as _; // global logger

use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f4xx_hal::pac;

#[entry]
fn main() -> ! {
    asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    let p = pac::Peripherals::take().unwrap();
    defmt::println!("Starting up");
    loop {
        // your code goes here
    }
}
