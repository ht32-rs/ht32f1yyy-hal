//! I2C example for the HT32F1755
#![no_std]
#![no_main]

use defmt_rtt as _;
use hal::hal as eh;
use ht32f1yyy_hal as hal;
use panic_probe as _;

use hal::ckcu::CkcuExt;
use hal::gpio::{Afio, GpioExt};
use hal::i2c::I2cExt;
use hal::pac;
use hal::time::RateExtU32;

use eh::i2c::{I2c, Operation};
use eh::digital::OutputPin;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: I2C");
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);
    let mut afio = Afio::new(dp.AFIO);

    let clocks = ckcu
        .configuration
        .use_hse(8.MHz())
        .ck_sys(144u32.MHz())
        .hclk(72u32.MHz())
        .ck_usb(48u32.MHz())
        .freeze();

    let gpioe = dp.GPIOE.split();

    let mut wp = gpioe.pe8.into_output_push_pull();
    let scl = gpioe
        .pe9
        .into_output_open_drain()
        .into_alternate_af3(&mut afio);
    let sda = gpioe
        .pe10
        .into_output_open_drain()
        .into_alternate_af3(&mut afio);

    let mut i2c = dp.I2C1.i2c(scl, sda, 400.kHz(), &clocks);
    let eeprom_addr = 0b1010000;
    wp.set_low().ok();

    // Read the contents of a 2Kibit (= 256 byte) eeprom
    let mut read_buf = [0u8; 8];
    for i in 0..32 {
        let addr = [i * 8];
        let mut operations = [Operation::Write(&addr), Operation::Read(&mut read_buf)];
        match i2c.transaction(eeprom_addr, &mut operations) {
            Ok(_) => defmt::info!("Eeprom page {}: {:x}", i, read_buf),
            Err(_) => defmt::warn!("Failed to read from eeprom"),
        }
    }

    loop {}
}
