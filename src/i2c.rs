//! Inter Integrated Circuit implementation
use crate::ckcu::{Clocks, Pcer};
use crate::gpio::{OpenDrain, Output};
use crate::hal::{self, i2c::Operation};
use crate::pac::{I2C0, I2C1};
use crate::Sealed;
use crate::time::{Hertz, RateExtU32};

use core::marker::PhantomData;
use core::ops::Deref;

pub use crate::hal::i2c::{AddressMode, SevenBitAddress, TenBitAddress};

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Arbitration error
    Arbitration,
    /// Bus error
    Bus,
    /// The slave didn't send ACK
    NotAcknowledge,
}

#[derive(Debug)]
pub enum Event {
    RxBufferFull,
    DataRegisterEmtpyTransmitter,
    DataRegisterEmptyReceiver,
    BusError,
    ReceivedNotAcknowledge,
    ArbitrationLoss,
    StopConditionDetected,
    StartConditionTransmit,
}

pub trait PinScl<I2C> {}

pub trait PinSda<I2C> {}

#[derive(Debug)]
pub struct I2c<I2C, ADRM: AddressMode> {
    i2c: I2C,
    addressing_mode: PhantomData<ADRM>,
}

pub trait I2cExt<I2C>: Sealed {
    fn i2c<SCL, SDA, F>(
        self,
        scl: SCL,
        sda: SDA,
        freq: F,
        clocks: &Clocks,
    ) -> I2c<I2C, SevenBitAddress>
    where
        SCL: PinScl<I2C>,
        SDA: PinSda<I2C>,
        F: Into<Hertz>;

    fn i2c_unchecked<F>(self, freq: F, clocks: &Clocks) -> I2c<I2C, SevenBitAddress>
    where
        F: Into<Hertz>;
}

impl Sealed for I2C0 {}

impl Sealed for I2C1 {}

macro_rules! i2c {
    ($($I2CX:ident,)+) => {
        $(
            impl I2c<$I2CX, SevenBitAddress>
            {
                /// Creates a new I2C peripheral
                pub fn new<F>(i2c: $I2CX, freq: F, clocks: &Clocks) -> Self
                where
                    F: Into<Hertz>,
                {
                    let freq = freq.into();

                    assert!(freq <= 1u32.MHz::<1, 1>());

                    // SCL_low = 1/pclk * (SLPG + d)
                    // SCL_high = 1/pclk * (SHPG + d)
                    // T_SCL = SCL_low + SCL_high
                    //
                    // For HT32F1yyy PCLK = HCLK
                    // The value of the `d` term depends on the device
                    #[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
                    let (d_l, d_h) = (9, 7);

                    #[cfg(any(
                        feature = "ht32f1653",
                        feature = "ht32f1654",
                        feature = "ht32f1655",
                        feature = "ht32f1656",
                    ))]
                    let (d_l, d_h) = (6, 6); // Depends on SEQ_FILTER, which we leave on 0b00

                    #[cfg(any(
                        feature = "ht32f1251",
                        feature = "ht32f1252",
                        feature = "ht32f1253",
                    ))]
                    let (d_l, d_h) = (7, 7);

                    let (shpg, slpg) = if freq > 100u32.kHz::<1, 1>() {
                        // We are in Fast-mode or Fast-mode Plus, this means
                        // SCL_low = 2 * SCL_high, refer to I2C spec page 48
                        // -> SCL_low = 2/3 SCL
                        // -> SLPG = (2 * PCLK) / (3 * SCL) - d_l
                        let slpg = ((2 * clocks.hclk.raw()) / (3 * freq.raw())) - d_l;

                        // 1/pclk * (SLPG + d_l) = 2/pclk * (SHPG + d_h)
                        // -> SHPG = (SLPG + d_l)/2 - d_h
                        // + 1 serves as a correction factor so SCL gets slower
                        // rather than larger as freq
                        let shpg = ((slpg + d_l) / 2) - d_h + 1;
                        (shpg, slpg)
                    } else {
                        // We are in Standard mode, this means
                        // SCL_low = SCL_high, refer to I2C spec page 48
                        // -> SLPG = SHPG = pclk / (2*SCL) - d
                        let scl_div = ((clocks.hclk.raw()) / (2 * freq.raw()));
                        (scl_div - d_h, scl_div - d_l)
                    };

                    // reset the I2C port before using it
                    i2c.reset();
                    // enable the AHB clock for the I2C port
                    i2c.enable();

                    // Configure the SCL clock values
                    i2c.i2c_shpgr
                        .modify(|_, w| unsafe { w.shpg().bits(shpg.try_into().unwrap()) });
                    i2c.i2c_slpgr
                        .modify(|_, w| unsafe { w.slpg().bits(slpg.try_into().unwrap()) });
                    // Enable the I2C port
                    i2c.i2c_cr.modify(|_, w| w.i2cen().set_bit());
                    I2c { i2c, addressing_mode: PhantomData }
                }
            }

            impl I2cExt<$I2CX> for $I2CX
            {
                fn i2c<SCL, SDA, F>(
                    self,
                    _scl: SCL,
                    _sda: SDA,
                    freq: F,
                    clocks: &Clocks
                ) -> I2c<$I2CX, SevenBitAddress>
                where
                    SCL: PinScl<$I2CX>,
                    SDA: PinSda<$I2CX>,
                    F: Into<Hertz>,
                {
                    I2c::<$I2CX, SevenBitAddress>::new(self, freq, clocks)
                }

                fn i2c_unchecked<F>(self, freq: F, clocks: &Clocks) -> I2c<$I2CX, SevenBitAddress>
                where
                    F: Into<Hertz>,
                {
                    I2c::<$I2CX, SevenBitAddress>::new(self, freq, clocks)
                }
            }
        )+
    }
}

i2c!(
    I2C0,
    I2C1,
);

macro_rules! busy_wait {
    ($i2c:expr, $field:ident, $variant:ident) => {
        loop {
            let status = $i2c.i2c_sr.read();

            if status.$field().$variant() {
                break;
            } else if status.arblos().bit_is_set() {
                return Err(Error::Arbitration);
            } else if status.rxnack().bit_is_set() {
                return Err(Error::NotAcknowledge);
            } else if status.buserr().bit_is_set() {
                return Err(Error::Bus);
            } else {
                // no error
            }
        }
    };
}

impl<I2C, ADRM> I2c<I2C, ADRM>
where
    I2C: Deref<Target = crate::pac::i2c0::RegisterBlock>,
    ADRM: AddressMode,
{
    pub fn free(self) -> I2C {
        self.i2c
    }

    pub fn listen(&mut self, event: Event) {
        match event {
            Event::RxBufferFull => self.i2c.i2c_ier.modify(|_, w| w.rxbfie().set_bit()),
            Event::DataRegisterEmtpyTransmitter => {
                self.i2c.i2c_ier.modify(|_, w| w.txdeie().set_bit())
            }
            Event::DataRegisterEmptyReceiver => {
                self.i2c.i2c_ier.modify(|_, w| w.rxdneie().set_bit())
            }
            Event::BusError => self.i2c.i2c_ier.modify(|_, w| w.buserrie().set_bit()),
            Event::ReceivedNotAcknowledge => self.i2c.i2c_ier.modify(|_, w| w.rxnackie().set_bit()),
            Event::ArbitrationLoss => self.i2c.i2c_ier.modify(|_, w| w.arblosie().set_bit()),
            Event::StopConditionDetected => self.i2c.i2c_ier.modify(|_, w| w.stoie().set_bit()),
            Event::StartConditionTransmit => self.i2c.i2c_ier.modify(|_, w| w.staie().set_bit()),
        }
    }

    pub fn unlisten(&mut self, event: Event) {
        match event {
            Event::RxBufferFull => self.i2c.i2c_ier.modify(|_, w| w.rxbfie().clear_bit()),
            Event::DataRegisterEmtpyTransmitter => {
                self.i2c.i2c_ier.modify(|_, w| w.txdeie().clear_bit())
            }
            Event::DataRegisterEmptyReceiver => {
                self.i2c.i2c_ier.modify(|_, w| w.rxdneie().clear_bit())
            }
            Event::BusError => self.i2c.i2c_ier.modify(|_, w| w.buserrie().clear_bit()),
            Event::ReceivedNotAcknowledge => {
                self.i2c.i2c_ier.modify(|_, w| w.rxnackie().clear_bit())
            }
            Event::ArbitrationLoss => self.i2c.i2c_ier.modify(|_, w| w.arblosie().clear_bit()),
            Event::StopConditionDetected => self.i2c.i2c_ier.modify(|_, w| w.stoie().clear_bit()),
            Event::StartConditionTransmit => self.i2c.i2c_ier.modify(|_, w| w.staie().clear_bit()),
        }
    }

    /// Set the target slave device address and wait for the start condition
    /// and the acknowledgement on the address frame.
    fn set_target_address(&mut self, addr: u16, operation: &Operation) -> Result<(), Error> {
        let rwd = match operation {
            Operation::Read(_) => true,
            Operation::Write(_) => false,
        };

        #[rustfmt::skip]
        self.i2c.i2c_tar.modify(|_, w| unsafe {
            // Set direction
            w.rwd().bit(rwd)
            // Set slave address with read bit
             .tar().bits(addr)
        });

        // wait for the start to be sent
        busy_wait!(self.i2c, sta, bit_is_set);
        // wait for the address frame to be sent and ACKed
        busy_wait!(self.i2c, adrs, bit_is_set);

        Ok(())
    }
}

impl<I2C> I2c<I2C, SevenBitAddress>
where
    I2C: Deref<Target = crate::pac::i2c0::RegisterBlock>,
{
    /// Change the addressing to 10-bit mode
    pub fn set_10bit_address(self) -> I2c<I2C, TenBitAddress> {
        self.i2c.i2c_cr.modify(|_, w| w.adrm().set_bit());
        I2c {
            i2c: self.i2c,
            addressing_mode: PhantomData,
        }
    }
}

impl<I2C> I2c<I2C, TenBitAddress>
where
    I2C: Deref<Target = crate::pac::i2c0::RegisterBlock>,
{
    /// Change the addressing to 7-bit mode
    pub fn set_7bit_address(self) -> I2c<I2C, SevenBitAddress> {
        self.i2c.i2c_cr.modify(|_, w| w.adrm().clear_bit());
        I2c {
            i2c: self.i2c,
            addressing_mode: PhantomData,
        }
    }
}

impl hal::i2c::Error for Error {
    fn kind(&self) -> hal::i2c::ErrorKind {
        match *self {
            Error::Arbitration => hal::i2c::ErrorKind::ArbitrationLoss,
            Error::Bus => hal::i2c::ErrorKind::Bus,
            Error::NotAcknowledge => {
                hal::i2c::ErrorKind::NoAcknowledge(hal::i2c::NoAcknowledgeSource::Unknown)
            }
        }
    }
}

impl<I2C, ADRM: AddressMode> hal::i2c::ErrorType for I2c<I2C, ADRM> {
    type Error = Error;
}

impl<I2C, ADRM> hal::i2c::I2c<ADRM> for I2c<I2C, ADRM>
where
    I2C: Deref<Target = crate::pac::i2c0::RegisterBlock>,
    ADRM: AddressMode + Into<u16>,
{
    fn transaction(
        &mut self,
        address: ADRM,
        operations: &mut [hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut set_tar = true;
        let mut peekable = operations.into_iter().peekable();
        let address = address.into();

        while let Some(operation) = peekable.next() {
            if set_tar {
                self.set_target_address(address, operation)?;
            }

            match operation {
                Operation::Read(buffer) => {
                    set_tar = match peekable.peek() {
                        Some(Operation::Write(_)) | None => true,
                        _ => false,
                    };

                    // configure acknowledgement to be sent upon reception
                    self.i2c.i2c_cr.modify(|_, w| w.aa().set_bit());

                    let last = buffer.len() - 1;
                    for (i, byte) in buffer.iter_mut().enumerate() {
                        if set_tar && i == last {
                            // send a NACK for the last byte,
                            // but only if this is the last read operation
                            self.i2c.i2c_cr.modify(|_, w| w.aa().clear_bit());
                        }
                        // wait until we received data
                        busy_wait!(self.i2c, rxdne, bit_is_set);
                        // read the byte
                        *byte = self.i2c.i2c_dr.read().data().bits();
                    }

                }
                Operation::Write(buffer) => {
                    set_tar = match peekable.peek() {
                        Some(Operation::Read(_)) | None => true,
                        _ => false,
                    };

                    for byte in buffer.iter() {
                        // wait for the byte to be sent and acked
                        busy_wait!(self.i2c, txde, bit_is_set);
                        // send the byte
                        self.i2c.i2c_dr.write(|w| unsafe { w.data().bits(*byte) });
                    }
                }
            }
        }

        // send the STOP
        self.i2c.i2c_cr.modify(|_, w| w.stop().set_bit());

        Ok(())
    }
}

macro_rules! pins {
    ($($I2CX:ty: SCL: [$($SCL:ty),*] SDA: [$($SDA:ty),*])+) => {
        $(
            $(
                impl PinScl<$I2CX> for $SCL {}
            )*
            $(
                impl PinSda<$I2CX> for $SDA {}
            )*
        )+
    }
}

#[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
use crate::gpio::{gpiob::*, gpioc::*, gpiod::*, gpioe::*, AF1, AF2, AF3};

#[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
pins! {
    I2C0:
        SCL: [
            PC4<Output<OpenDrain>, AF2>,
            PC11<Output<OpenDrain>, AF1>,
            PD12<Output<OpenDrain>, AF2>
        ]

        SDA: [
            PC5<Output<OpenDrain>, AF2>,
            PC12<Output<OpenDrain>, AF1>,
            PD13<Output<OpenDrain>, AF2>
        ]

    I2C1:
        SCL: [
            PC0<Output<OpenDrain>, AF3>,
            PC6<Output<OpenDrain>, AF1>,
            PE9<Output<OpenDrain>, AF3>
        ]

        SDA: [
            PB7<Output<OpenDrain>, AF2>,
            PC1<Output<OpenDrain>, AF3>,
            PC7<Output<OpenDrain>, AF1>,
            PE10<Output<OpenDrain>, AF3>
        ]
}

// TODO: pins! for other devices
