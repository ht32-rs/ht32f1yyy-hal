//! USB serial example for the HT32F1755

#![no_std]
#![no_main]

use defmt_rtt as _;
use ht32f1yyy_hal as hal;
use panic_probe as _;

use hal::ckcu::CkcuExt;
use hal::gpio::GpioExt;
use hal::pac;
use hal::time::RateExtU32;
use hal::usb::{Peripheral, UsbBus};

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Example: USB serial");
    let dp = pac::Peripherals::take().unwrap();
    let ckcu = dp.CKCU.constrain(dp.RSTCU);

    let _clocks = ckcu
        .configuration
        .use_hse(8.MHz())
        .ck_sys(144u32.MHz())
        .hclk(72u32.MHz())
        .ck_usb(48u32.MHz())
        .freeze();

    let gpioa = dp.GPIOA.split();
    let dppu = gpioa.pa4.into_output_push_pull();

    let usb = Peripheral { usb: dp.USB, dppu };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(USB_CLASS_CDC)
        .strings(&[StringDescriptors::new(LangID::EN)
            .manufacturer("BigCo Inc.")
            .product("Serial port")
            .serial_number("DEADBEEF")])
        .expect("Cannot set string descriptors")
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
