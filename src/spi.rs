//! Serial Peripheral Interface implementation
use crate::ckcu::{Clocks, Pcer};
use crate::gpio::{Floating, Input, Output, PushPull};
use crate::hal;
use crate::pac::{SPI0, SPI1};
use crate::time::Hertz;
use crate::Sealed;

use core::marker::PhantomData;
use core::ops::Deref;

pub use crate::hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Write Collision occured
    WriteCollision,
}

#[derive(Debug)]
pub enum Event {
    ModeFault,
    ReadOverrun,
    WriteCollision,
    RxBufferNotEmpty,
    TxEmpty,
    TxBufferEmpty,
}

pub trait PinSck<SPI> {}
pub trait PinMiso<SPI> {}
pub trait PinMosi<SPI> {}

#[derive(Debug)]
pub struct Spi<SPI, WORD = u8> {
    spi: SPI,
    _word: PhantomData<WORD>,
}

// TODO: also allow creation with chip-select pin
// and implement SpiDevice trait instead of SpiBus in that case
pub trait SpiExt<SPI, WORD>: Sealed {
    fn spi<SCK, MISO, MOSI, F>(
        self,
        sck: SCK,
        miso: MISO,
        mosi: MOSI,
        mode: Mode,
        freq: F,
        clocks: &Clocks,
    ) -> Spi<SPI, WORD>
    where
        SCK: PinSck<SPI>,
        MISO: PinMiso<SPI>,
        MOSI: PinMosi<SPI>,
        F: Into<Hertz>;

    fn spi_unchecked<F>(self, mode: Mode, freq: F, clocks: &Clocks) -> Spi<SPI, WORD>
    where
        F: Into<Hertz>;
}

impl Sealed for SPI0 {}

impl Sealed for SPI1 {}

macro_rules! spi {
    ($($SPIX:ident => ($($WORD:ident),+),)+) => {
        $($(
            impl Spi<$SPIX, $WORD> {
                fn new<F>(spi: $SPIX, mode: Mode, freq: F, clocks: &Clocks) -> Spi<$SPIX, $WORD>
                where
                    F: Into<Hertz>,
                {
                    // reset the SPI port before using it
                    spi.reset();
                    // enable the APB clock for the SPI port
                    spi.enable();

                    // Cfr. SPICR1 FORMAT table in User Manual
                    let cpol = (mode.polarity == Polarity::IdleHigh) as u8;
                    let cpha = (mode.phase == Phase::CaptureOnSecondTransition) as u8;
                    let mode = (cpol << 2) | ((cpol ^ cpha) << 1) | (!(cpol ^ cpha));

                    let word = ((core::mem::size_of::<$WORD>() * 8) & 0xFF) as u8;

                    #[rustfmt::skip]
                    spi.spi_cr1.modify(|_, w| unsafe {
                        w.mode().set_bit() // master mode
                         .selm().clear_bit() // software SS
                         .firstbit().clear_bit() // MSB first
                         .format().bits(mode) // SPI mode
                         .dfl().bits(word) // data frame length
                    });

                    // f_sck = f_pclk / (2 *  (CP + 1)) according to User Manual
                    // -> CP = (f_pclk / (2 * f_sck)) - 1
                    // for pclk = hclk
                    let freq = freq.into();
                    let spi_div: u16 = ((clocks.hclk.raw() / (2 * freq.raw())) - 1) as u16;

                    spi.spi_cpr.write(|w| unsafe { w.cp().bits(spi_div) });

                    // Select pin output enable
                    // This causes the chip to not mode fault all the time
                    // when it's not in a multi master setup.
                    spi.spi_cr0.modify(|_, w| w.seloen().set_bit());

                    spi.spi_cr0.modify(|_, w| w.spien().set_bit());
                    Spi {
                        spi,
                        _word: PhantomData,
                    }
                }
            }

            impl SpiExt<$SPIX, $WORD> for $SPIX
            {
                fn spi<SCK, MISO, MOSI, F>(
                    self,
                    _sck: SCK,
                    _miso: MISO,
                    _mosi: MOSI,
                    mode: Mode,
                    freq: F,
                    clocks: &Clocks,
                ) -> Spi<$SPIX, $WORD>
                where
                    SCK: PinSck<$SPIX>,
                    MISO: PinMiso<$SPIX>,
                    MOSI: PinMosi<$SPIX>,
                    F: Into<Hertz>,
                {
                    Spi::<$SPIX, $WORD>::new(self, mode, freq, clocks)
                }

                fn spi_unchecked<F>(self, mode: Mode, freq: F, clocks: &Clocks) -> Spi<$SPIX, $WORD>
                where
                    F: Into<Hertz>,
                {
                    Spi::<$SPIX, $WORD>::new(self, mode, freq, clocks)
                }
            }
        )+)+
    }
}

spi! {
    SPI0 => (u8, u16),
    SPI1 => (u8, u16),
}

impl<SPI, WORD> Spi<SPI, WORD>
where
    SPI: Deref<Target = crate::pac::spi0::RegisterBlock>,
{
    pub fn free(self) -> SPI {
        self.spi
    }

    pub fn listen(&mut self, event: Event) {
        match event {
            Event::ModeFault => self.spi.spi_ier.modify(|_, w| w.mfien().set_bit()),
            Event::ReadOverrun => self.spi.spi_ier.modify(|_, w| w.roien().set_bit()),
            Event::WriteCollision => self.spi.spi_ier.modify(|_, w| w.wcien().set_bit()),
            Event::RxBufferNotEmpty => self.spi.spi_ier.modify(|_, w| w.rxbneien().set_bit()),
            Event::TxEmpty => self.spi.spi_ier.modify(|_, w| w.txeien().set_bit()),
            Event::TxBufferEmpty => self.spi.spi_ier.modify(|_, w| w.txbeien().set_bit()),
        }
    }
    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::ModeFault => self.spi.spi_ier.modify(|_, w| w.mfien().clear_bit()),
            Event::ReadOverrun => self.spi.spi_ier.modify(|_, w| w.roien().clear_bit()),
            Event::WriteCollision => self.spi.spi_ier.modify(|_, w| w.wcien().clear_bit()),
            Event::RxBufferNotEmpty => self.spi.spi_ier.modify(|_, w| w.rxbneien().clear_bit()),
            Event::TxEmpty => self.spi.spi_ier.modify(|_, w| w.txeien().clear_bit()),
            Event::TxBufferEmpty => self.spi.spi_ier.modify(|_, w| w.txbeien().clear_bit()),
        }
    }

    #[inline(always)]
    fn read_nonblocking(&mut self) -> nb::Result<WORD, Error> {
        let sr = self.spi.spi_sr.read();

        Err(if sr.ro().bit_is_set() {
            Error::Overrun.into()
        } else if sr.wc().bit_is_set() {
            Error::WriteCollision.into()
        } else if sr.rxbne().bit_is_set() {
            return Ok(unsafe {
                core::ptr::read_volatile(&self.spi.spi_dr as *const _ as *const WORD)
            });
        } else {
            nb::Error::WouldBlock
        })
    }

    #[inline(always)]
    fn write_nonblocking(&mut self, byte: WORD) -> nb::Result<(), Error> {
        let sr = self.spi.spi_sr.read();

        Err(if sr.ro().bit_is_set() {
            Error::Overrun.into()
        } else if sr.wc().bit_is_set() {
            Error::WriteCollision.into()
        } else if sr.txe().bit_is_set() {
            unsafe { core::ptr::write_volatile(&self.spi.spi_dr as *const _ as *mut WORD, byte) }
            return Ok(());
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl hal::spi::Error for Error {
    fn kind(&self) -> hal::spi::ErrorKind {
        match *self {
            Error::Overrun => hal::spi::ErrorKind::Overrun,
            Error::WriteCollision => hal::spi::ErrorKind::Other,
        }
    }
}

impl<SPI, WORD> hal::spi::ErrorType for Spi<SPI, WORD> {
    type Error = Error;
}

impl<SPI, Word: Copy + Default + 'static> hal::spi::SpiBus<Word> for Spi<SPI, Word>
where
    SPI: Deref<Target = crate::pac::spi0::RegisterBlock>,
{
    fn read(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
        for word in words {
            nb::block!(self.write_nonblocking(Word::default()))?;
            *word = nb::block!(self.read_nonblocking())?;
        }

        Ok(())
    }

    fn write(&mut self, words: &[Word]) -> Result<(), Self::Error> {
        for word in words {
            nb::block!(self.write_nonblocking(*word))?;
            nb::block!(self.read_nonblocking())?;
        }

        Ok(())
    }

    fn transfer(&mut self, read: &mut [Word], write: &[Word]) -> Result<(), Self::Error> {
        assert_eq!(write.len(), read.len());

        for (d, b) in write.iter().cloned().zip(read.iter_mut()) {
            nb::block!(self.write_nonblocking(d))?;
            *b = nb::block!(self.read_nonblocking())?;
        }

        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
        for word in words {
            nb::block!(self.write_nonblocking(*word))?;
            *word = nb::block!(self.read_nonblocking())?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

macro_rules! pins {
    ($($SPIX:ty: SCK: [$($SCK:ty),*] MISO: [$($MISO:ty),*] MOSI: [$($MOSI:ty),*])+) => {
        $(
            $(
                impl PinSck<$SPIX> for $SCK {}
            )*
            $(
                impl PinMiso<$SPIX> for $MISO {}
            )*
            $(
                impl PinMosi<$SPIX> for $MOSI {}
            )*
        )+
    }
}

#[cfg(any(feature = "ht32f1653", feature = "ht32f1654"))]
use crate::gpio::{gpioa::*, gpiob::*, gpioc::*, gpiod::*, AF5};

#[cfg(any(feature = "ht32f1653", feature = "ht32f1654"))]
pins! {
    SPI0:
        SCK: [
            PA4<Output<PushPull>, AF5>,
            PB3<Output<PushPull>, AF5>,
            PD2<Output<PushPull>, AF5>
        ]
        MISO: [
            PA6<Input<Floating>, AF5>,
            PA11<Input<Floating>, AF5>,
            PB5<Input<Floating>, AF5>
        ]
        MOSI: [
            PA5<Output<PushPull>, AF5>,
            PA9<Output<PushPull>, AF5>,
            PB4<Output<PushPull>, AF5>
        ]
    SPI1:
        SCK: [
            PA15<Output<PushPull>, AF5>,
            PB7<Output<PushPull>, AF5>,
            PC1<Output<PushPull>, AF5>,
            PC11<Output<PushPull>, AF5>
        ]
        MISO: [
            PB1<Input<Floating>, AF5>,
            PB9<Input<Floating>, AF5>,
            PC3<Input<Floating>, AF5>,
            PC12<Input<Floating>, AF5>
        ]
        MOSI: [
            PB0<Output<PushPull>, AF5>,
            PB8<Output<PushPull>, AF5>,
            PC2<Output<PushPull>, AF5>,
            PC11<Output<PushPull>, AF5>
        ]
}

// TODO: pins! for other devices
