//! HT32F1yyy hardware abstraction layer.

#![cfg_attr(not(test), no_std)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires one of the following device features enabled:
        ht32f1251
        ht32f1252
        ht32f1253
        ht32f1653
        ht32f1654
        ht32f1655
        ht32f1656
        ht32f1755
        ht32f1765
"
);

pub use embedded_hal as hal;

#[cfg(any(
    feature = "ht32f1251",
    feature = "ht32f1252",
    feature = "ht32f1253",
))]
pub use ht32f1yyy::ht32f125x as pac;

#[cfg(any(
    feature = "ht32f1653",
    feature = "ht32f1654",
))]
pub use ht32f1yyy::ht32f1653_54 as pac;

#[cfg(any(
    feature = "ht32f1655",
    feature = "ht32f1656",
))]
pub use ht32f1yyy::ht32f1655_56 as pac;

#[cfg(any(
    feature = "ht32f1755",
    feature = "ht32f1765",
))]
pub use ht32f1yyy::ht32f175x as pac;

#[cfg(feature = "rt")]
pub use crate::pac::interrupt;

pub mod ckcu;
pub mod gpio;
pub mod i2c;
pub mod time;

#[cfg(not(any(
    feature = "ht32f1251",
    feature = "ht32f1252",
    feature = "ht32f1253",
)))]
pub mod usb;

mod sealed {
    pub trait Sealed {}
}
pub(crate) use sealed::Sealed;

