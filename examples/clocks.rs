//! Clock configuration example
#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use ht32f1yyy_hal as hal;

use hal::pac;
use hal::ckcu::CkcuExt;
use hal::time::RateExtU32;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: CKCU");
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);

    ckcu.configuration.ck_sys(72u32.MHz()).freeze();

    defmt::info!("Example: CKCU, done");
    loop {}
}
