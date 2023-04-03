//! HT32F1yyy hardware abstraction layer.

#![cfg_attr(not(test), no_std)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires one of the following device features enabled:
        ht32f125x
        ht32f1653_54
        ht32f1655_56
        ht32f175x
"
);

pub use embedded_hal as hal;

#[cfg(any(feature = "ht32f125x",))]
pub use ht32f1yyy::ht32f125x as ht32;

#[cfg(any(feature = "ht32f1653_54",))]
pub use ht32f1yyy::ht32f1653_54 as ht32;

#[cfg(any(feature = "ht32f1655_56",))]
pub use ht32f1yyy::ht32f1655_56 as ht32;

#[cfg(any(feature = "ht32f175x",))]
pub use ht32f1yyy::ht32f175x as ht32;

pub mod ckcu;
pub mod gpio;
pub mod time;
