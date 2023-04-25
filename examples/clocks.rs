//! Clock configuration example
#![no_std]
#![no_main]

use defmt_rtt as _;
use ht32f1yyy_hal as hal;
use panic_probe as _;

use hal::ckcu::CkcuExt;
use hal::pac;
use hal::time::RateExtU32;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: CKCU");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);

    ckcu.configuration
        .use_hse(8.MHz())
        .ck_sys(144u32.MHz())
        .hclk(72u32.MHz())
        .ck_usb(48u32.MHz())
        .freeze();

    defmt::info!("Example: CKCU, done");

    let mut syst = cp.SYST;
    // SysTick happens every 8 core clock cycles
    // reload value = 72 MHz / 8 -> 1 Hz wrap
    syst.set_reload(9_000_000);
    syst.clear_current();
    syst.enable_counter();

    loop {
        if syst.has_wrapped() {
            defmt::info!("Tick");
        }
    }
}
