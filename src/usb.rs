//! USB peripheral implementation for HT32F1yyy microcontrollers.

pub use ht32_usbd::UsbBus;
use ht32_usbd::UsbPeripheral;

use crate::ckcu::Pcer;
#[cfg(not(feature = "dppu"))]
use crate::hal::digital::OutputPin;
use crate::pac::USB;

#[cfg(feature = "dppu")]
pub struct Peripheral {
    pub usb: USB,
}
#[cfg(feature = "dppu")]
unsafe impl Sync for Peripheral {}

#[cfg(feature = "dppu")]
unsafe impl UsbPeripheral for Peripheral {
    const EP_MEMORY: *const () = 0x400A_A000 as _;
    const EP_MEMORY_SIZE: usize = 1024;
    const DP_PULL_UP_FEATURE: bool = true;
    const REGISTERS: *const () = USB::ptr() as *const ();

    fn enable(&self) {
        self.usb.enable();
    }

    fn dp_pull_up(&mut self) {}
}

#[cfg(not(feature = "dppu"))]
pub struct Peripheral<PIN: OutputPin + Sync> {
    pub usb: USB,
    pub dppu: PIN,
}

#[cfg(not(feature = "dppu"))]
unsafe impl<PIN: OutputPin + Sync> Sync for Peripheral<PIN> {}

#[cfg(not(feature = "dppu"))]
unsafe impl<PIN: OutputPin + Sync + Send> UsbPeripheral for Peripheral<PIN> {
    const EP_MEMORY: *const () = 0x4004_E400 as _;
    const EP_MEMORY_SIZE: usize = 1024;
    const DP_PULL_UP_FEATURE: bool = false;
    const REGISTERS: *const () = USB::ptr() as *const ();

    fn enable(&self) {
        self.usb.enable();
    }

    fn dp_pull_up(&mut self) {
        self.dppu.set_high().ok();
    }
}
