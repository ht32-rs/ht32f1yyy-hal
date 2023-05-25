use crate::pac::usb::RegisterBlock;
use super::UsbPeripheral;
use core::marker::PhantomData;

/// A proxy type that provides unified register interface
pub struct UsbRegisters<USB> {
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> core::ops::Deref for UsbRegisters<USB> {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*crate::pac::USB::ptr() }
    }
}

impl<USB: UsbPeripheral> UsbRegisters<USB> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}
