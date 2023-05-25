//! USB peripheral driver for HT32 microcontrollers.

pub mod bus;
mod endpoint_memory;
mod endpoints;
mod registers;
pub use bus::UsbBus;

use crate::ckcu::Pcer;
use crate::hal::digital::OutputPin;
use crate::pac::USB;

/// A trait for device-specific USB peripherals. Implement this to add support for a new hardware
/// platform. Peripherals that have this trait must have the same register block as HT32 USBFS
/// peripherals.
pub unsafe trait UsbPeripheral: Send + Sync {
    /// Pointer to the endpoint memory
    const EP_MEMORY: *const ();

    /// Endpoint memory size in bytes
    const EP_MEMORY_SIZE: usize;

    /// Enables USB device on its peripheral bus
    fn enable(&self);

    #[cfg(not(feature = "dppu"))]
    /// Enable the pull-up on the DP line.
    fn dp_pull_up(&mut self);
}

pub struct Peripheral<PIN: OutputPin + Sync> {
    pub usb: USB,
    #[cfg(not(feature = "dppu"))]
    pub dppu: PIN,
}

unsafe impl<PIN: OutputPin + Sync> Sync for Peripheral<PIN> {}

unsafe impl<PIN: OutputPin + Sync + Send> UsbPeripheral for Peripheral<PIN> {
    #[cfg(any(
        feature = "ht32f1653",
        feature = "ht32f1654",
        feature = "ht32f1655",
        feature = "ht32f1656",
    ))]
    const EP_MEMORY: *const () = 0x400A_A000 as _;
    #[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
    const EP_MEMORY: *const () = 0x4004_E400 as _;
    const EP_MEMORY_SIZE: usize = 1024;

    fn enable(&self) {
        self.usb.enable();
    }

    #[cfg(not(feature = "dppu"))]
    fn dp_pull_up(&mut self) {
        self.dppu.set_high().ok();
    }
}
