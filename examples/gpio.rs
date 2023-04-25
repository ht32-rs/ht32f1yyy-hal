//! Clock configuration example
#![no_std]
#![no_main]

use defmt_rtt as _;
use ht32f1yyy_hal as hal;
use panic_probe as _;

use hal::gpio::Afio;
use hal::ckcu::CkcuExt;
use hal::gpio::GpioExt;
use hal::pac;
use hal::time::RateExtU32;

use embedded_hal::digital::{InputPin, OutputPin};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: GPIO");
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);
    let mut afio = Afio::new(dp.AFIO);

    ckcu.configuration
        .use_hse(8.MHz())
        .ck_sys(144u32.MHz())
        .hclk(72u32.MHz())
        .ck_usb(48u32.MHz())
        .freeze();

    let gpioa = dp.GPIOA.split();
    let mut led0 = gpioa.pa0.into_output_push_pull();
    let input0 = gpioa.pa1.into_input_pull_down();
    let _test_af = gpioa.pa2.into_alternate_af2(&mut afio);
    led0.set_high().unwrap();

    loop {
        defmt::info!("input0, reading: {}", input0.is_high().unwrap());
    }
}
