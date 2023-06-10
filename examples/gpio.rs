//! GPIO example for the HT32F1755
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
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);
    let mut afio = Afio::new(dp.AFIO);

    let _clocks = ckcu.configuration
        .use_hse(8.MHz())
        .ck_sys(144u32.MHz())
        .hclk(72u32.MHz())
        .ck_usb(48u32.MHz())
        .freeze();

    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpioe = dp.GPIOE.split();

    // To access PB6 as GPIO, we have to enable AF1 on the pin
    let mut output = gpiob.pb6.into_alternate_af1(&mut afio).into_output_push_pull();
    let input = gpioc.pc11.into_input_pull_down();

    output.set_high().unwrap();

    let mut syst = cp.SYST;
    syst.set_reload(9_000_000);
    syst.clear_current();
    syst.enable_counter();

    loop {
        if syst.has_wrapped() {
            defmt::info!("input, reading: {}", input.is_high().unwrap());
        }
    }
}
