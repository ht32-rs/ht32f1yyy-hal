//! SPI example for the HT32F1654
#![no_std]
#![no_main]

use defmt_rtt as _;
use hal::hal as eh;
use ht32f1yyy_hal as hal;
use panic_probe as _;

use hal::ckcu::CkcuExt;
use hal::gpio::{Afio, GpioExt};
use hal::pac;
use hal::spi::SpiExt;
use hal::time::RateExtU32;

use eh::spi::{Operation, SpiDevice, MODE_0};
use embedded_hal_bus::spi::ExclusiveDevice;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: SPI");
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

    let gpiob = dp.GPIOB.split();

    let sck = gpiob
        .pb7
        .into_output_push_pull()
        .into_alternate_af5(&mut afio);
    let mosi = gpiob
        .pb8
        .into_output_push_pull()
        .into_alternate_af5(&mut afio);
    let miso = gpiob
        .pb9
        .into_input_floating()
        .into_alternate_af5(&mut afio);
    let cs = gpiob.pb10.into_output_push_pull();

    let spi: hal::spi::Spi<_, u8> = dp.SPI1.spi(sck, miso, mosi, MODE_0, 1.MHz(), &clocks);
    let mut device = ExclusiveDevice::new_no_delay(spi, cs);

    defmt::info!("Interact with SPI flash");

    // send Write Enable
    let cmd = [0x06];
    let mut operations = [Operation::Write(&cmd)];
    match device.transaction(&mut operations) {
        Ok(_) => defmt::info!("WREN success"),
        Err(_) => defmt::error!("WREN failure"),
    }

    // read Status Register
    let mut read_buf = [0u8];
    let cmd = [0x05];
    let mut operations = [
        Operation::Write(&cmd),
        Operation::Read(&mut read_buf),
    ];
    match device.transaction(&mut operations) {
        Ok(_) => defmt::info!("Read: {:?}", read_buf),
        Err(_) => defmt::error!("Failed to read"),
    }

    loop {}
}
