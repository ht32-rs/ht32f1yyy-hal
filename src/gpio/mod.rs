//! General Purpose Input / Output

mod afio;
pub use afio::Afio;

use core::convert::Infallible;
use core::marker::PhantomData;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

// Implemented by each GPIO register block to enable erased pins
trait GpioRegExt {
    fn is_low(&self, pos: u8) -> bool;
    fn is_set_low(&self, pos: u8) -> bool;
    fn set_high(&self, pos: u8);
    fn set_low(&self, pos: u8);
}

/// Output mode (type state)
/// `MODE`: describes the output type
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Open drain output (type state)
pub struct OpenDrain;

/// Push pull output (type state)
pub struct PushPull;

/// Input mode (type state)
/// `MODE`: describes the output type
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Pulled up input (type state)
pub struct PullUp;

/// Pulled down input (type state)
pub struct PullDown;

/// FLoating input (type state)
pub struct Floating;

/// Disabled input (type state)
/// Holtek chips do allow a chip to be configured as input but not actually read
/// any data.
pub struct Disabled;

/// Alternate function 0 (type state)
pub struct AF0;
/// Alternate function 1 (type state)
pub struct AF1;
/// Alternate function 2 (type state)
pub struct AF2;
/// Alternate function 3 (type state)
pub struct AF3;
/// Alternate function 4 (type state)
pub struct AF4;
/// Alternate function 5 (type state)
pub struct AF5;
/// Alternate function 6 (type state)
pub struct AF6;
/// Alternate function 7 (type state)
pub struct AF7;
/// Alternate function 8 (type state)
pub struct AF8;
/// Alternate function 9 (type state)
pub struct AF9;
/// Alternate function 10 (type state)
pub struct AF10;
/// Alternate function 11 (type state)
pub struct AF11;
/// Alternate function 12 (type state)
pub struct AF12;
/// Alternate function 13 (type state)
pub struct AF13;
/// Alternate function 14 (type state)
pub struct AF14;
/// Alternate function 15 (type state)
pub struct AF15;

/// The 4 current values that can be used for output pins
/// TODO: Migrate these into the PAC and re-export them here in order to avoid
/// API breaking.
#[derive(Copy, Clone, Debug)]
pub enum GpioCurrent {
    MA4,
    MA8,
}

impl GpioCurrent {
    // TODO
    #[allow(dead_code)]
    fn to_bits(self) -> u8 {
        match self {
            Self::MA4 => 0b0,
            Self::MA8 => 0b1,
        }
    }
}

/// Fully erased pin
pub struct Pin<MODE> {
    i: u8,
    port: *const dyn GpioRegExt,
    _mode: PhantomData<MODE>,
}

// NOTE(unsafe) The only write access is to SRR, which is thread safe
unsafe impl<MODE> Sync for Pin<MODE> {}
// NOTE(unsafe) this only enables read access to the same pin from multiple threads
unsafe impl<MODE> Send for Pin<MODE> {}

use crate::hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_set_low(self.i) })
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_high(self.i) };
        Ok(())
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.port).set_low(self.i) }
        Ok(())
    }
}

impl<MODE> ToggleableOutputPin for Pin<Output<MODE>> {
    fn toggle(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl InputPin for Pin<Output<OpenDrain>> {
    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { (*self.port).is_low(self.i) })
    }
}

impl<MODE> ErrorType for Pin<MODE> {
    type Error = Infallible;
}

// TODO: change the register names in the PAC, so we don't have to pass DINR, ODR, SRR, RR
macro_rules! gpio_trait {
    ($gpiox:ident : $gpiox_dinr:ident, $gpiox_doutr:ident, $gpiox_srr:ident, $gpiox_rr:ident) => {
        impl GpioRegExt for crate::pac::$gpiox::RegisterBlock {
            fn is_low(&self, pos: u8) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                self.$gpiox_dinr.read().bits() & (1 << pos) == 0
            }

            fn is_set_low(&self, pos: u8) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                self.$gpiox_doutr.read().bits() & (1 << pos) == 0
            }

            fn set_high(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.$gpiox_srr.write(|w| w.bits(1 << pos)) };
            }

            fn set_low(&self, pos: u8) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { self.$gpiox_rr.write(|w| w.bits(1 << pos)) };
            }
        }
    };
}

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $pxrst:ident, $pxen:ident, $gpiox_drvr:ident, $gpiox_dircr:ident, $gpiox_pur:ident, $gpiox_pdr:ident, $gpiox_iner: ident, $gpiox_odr:ident, [
         $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $AF:ty, $dirx:ident, $pux: ident, $pdx:ident, $inenx:ident, $odx:ident),)+
    ]) => {
        pub mod $gpiox {
            use core::convert::Infallible;
            use core::marker::PhantomData;

            use crate::hal::digital::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin, ErrorType};
            use crate::pac::$GPIOX;
            use crate::ckcu::Pcer;

            use crate::gpio::{Output, Input, OpenDrain, PushPull, PullDown, PullUp, Floating, Disabled};
            use crate::gpio::{GpioExt, GpioRegExt, Pin};
            use crate::gpio::afio::{Afio, AfioCfg};

            use crate::gpio::{AF0, AF1, AF2, AF3};
            #[cfg(not(feature = "afio4"))]
            use crate::gpio::{AF4, AF5, AF6, AF7, AF8, AF9, AF10, AF11, AF12, AF13, AF14, AF15};


            /// The to split the GPIO into
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE, $AF>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    self.reset();
                    self.enable();

                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData, _af: PhantomData },
                        )+
                    }
                }
            }

            // This is where all pins of this GPIO block as well as the GPIO state
            // machine is actually implemented.
            $(
                /// Pin
                pub struct $PXi<MODE, AF> {
                    _mode: PhantomData<MODE>,
                    _af: PhantomData<AF>
                }

                // These state transitions should be possible for any pin
                impl<MODE, AF> $PXi<MODE, AF> {
                    /// Change the pin to an output pin in push pull mode
                    pub fn into_output_push_pull(self) -> $PXi<Output<PushPull>, AF> {
                        // Set the direction to output
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_dircr.modify(|_, w| w.$dirx().set_bit());
                        // Disable open drain -> implcitly enable push pull
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_odr.modify(|_, w| w.$odx().clear_bit());

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the pin into an output pin in open drain mode
                    pub fn into_output_open_drain(self) -> $PXi<Output<OpenDrain>, AF> {
                        // Set the direction to output
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_dircr.modify(|_, w| w.$dirx().set_bit());
                        // Enable open drain
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_odr.modify(|_, w| w.$odx().set_bit());

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the pin into an input pin in pull up mode
                    pub fn into_input_pull_up(self) -> $PXi<Input<PullUp>, AF> {
                        // Set the direction to input
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_dircr.modify(|_, w| w.$dirx().clear_bit());
                        // Enable pull up
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_pur.modify(|_, w| w.$pux().set_bit());
                        // Enable the input function, this is what allows us to actually
                        // read values from the Schmitt trigger inside the GPIO circuit
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_iner.modify(|_, w| w.$inenx().set_bit());

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the pin into an input pin in pull down mode.
                    pub fn into_input_pull_down(self) -> $PXi<Input<PullDown>, AF> {
                        // Set the direction to input
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_dircr.modify(|_, w| w.$dirx().clear_bit());
                        // According to User Manual page 133 pull up takes priority over pull down,
                        // hence we have to disable it here explicitly
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_pur.modify(|_, w| w.$pux().clear_bit());
                        // Enable pull down
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_pdr.modify(|_, w| w.$pdx().set_bit());
                        // Enable the input function, this is what allows us to actually
                        // read values from the Schmitt trigger inside the GPIO circuit
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_iner.modify(|_, w| w.$inenx().set_bit());

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the pin into an input pin in floating mode
                    pub fn into_input_floating(self) -> $PXi<Input<Floating>, AF> {
                        // Set the direction to input
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_dircr.modify(|_, w| w.$dirx().clear_bit());
                        // Disable pull up
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_pur.modify(|_, w| w.$pux().clear_bit());
                        // Disable pull down
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_pdr.modify(|_, w| w.$pdx().clear_bit());
                        // Enable the input function, this is what allows us to actually
                        // read values from the Schmitt trigger inside the GPIO circuit
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_iner.modify(|_, w| w.$inenx().set_bit());

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF0, leave the IO mode alone though
                    pub fn into_alternate_af0(self, afio: &mut Afio) -> $PXi<MODE, AF0> {
                        self.into_alternate(afio, 0b00);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF1, leave the IO mode alone though
                    pub fn into_alternate_af1(self, afio: &mut Afio) -> $PXi<MODE, AF1> {
                        self.into_alternate(afio, 0b01);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF2, leave the IO mode alone though
                    pub fn into_alternate_af2(self, afio: &mut Afio) -> $PXi<MODE, AF2> {
                        self.into_alternate(afio, 0b10);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF3, leave the IO mode alone though
                    pub fn into_alternate_af3(self, afio: &mut Afio) -> $PXi<MODE, AF3> {
                        self.into_alternate(afio, 0b11);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }
                }

                #[cfg(not(feature = "afio4"))]
                impl<MODE, AF> $PXi<MODE, AF> {
                    /// Change the AF to AF4, leave the IO mode alone though
                    pub fn into_alternate_af4(self, afio: &mut Afio) -> $PXi<MODE, AF4> {
                        self.into_alternate(afio, 0b0100);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF5, leave the IO mode alone though
                    pub fn into_alternate_af5(self, afio: &mut Afio) -> $PXi<MODE, AF5> {
                        self.into_alternate(afio, 0b0101);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF6, leave the IO mode alone though
                    pub fn into_alternate_af6(self, afio: &mut Afio) -> $PXi<MODE, AF6> {
                        self.into_alternate(afio, 0b0110);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF7, leave the IO mode alone though
                    pub fn into_alternate_af7(self, afio: &mut Afio) -> $PXi<MODE, AF7> {
                        self.into_alternate(afio, 0b0111);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF8, leave the IO mode alone though
                    pub fn into_alternate_af8(self, afio: &mut Afio) -> $PXi<MODE, AF8> {
                        self.into_alternate(afio, 0b1000);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF9, leave the IO mode alone though
                    pub fn into_alternate_af9(self, afio: &mut Afio) -> $PXi<MODE, AF9> {
                        self.into_alternate(afio, 0b1001);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF10, leave the IO mode alone though
                    pub fn into_alternate_af10(self, afio: &mut Afio) -> $PXi<MODE, AF10> {
                        self.into_alternate(afio, 0b1010);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF11, leave the IO mode alone though
                    pub fn into_alternate_af11(self, afio: &mut Afio) -> $PXi<MODE, AF11> {
                        self.into_alternate(afio, 0b1011);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF12, leave the IO mode alone though
                    pub fn into_alternate_af12(self, afio: &mut Afio) -> $PXi<MODE, AF12> {
                        self.into_alternate(afio, 0b1100);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF13, leave the IO mode alone though
                    pub fn into_alternate_af13(self, afio: &mut Afio) -> $PXi<MODE, AF13> {
                        self.into_alternate(afio, 0b1101);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF14, leave the IO mode alone though
                    pub fn into_alternate_af14(self, afio: &mut Afio) -> $PXi<MODE, AF14> {
                        self.into_alternate(afio, 0b1110);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF15, leave the IO mode alone though
                    pub fn into_alternate_af15(self, afio: &mut Afio) -> $PXi<MODE, AF15> {
                        self.into_alternate(afio, 0b1111);
                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }
                }

                impl<MODE, AF> $PXi<MODE, AF> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> Pin<MODE> {
                        Pin {
                            i: $i,
                            port: $GPIOX::ptr() as *const dyn GpioRegExt,
                            _mode: self._mode,
                        }
                    }
                }

                impl<OUTPUT, AF> OutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        Ok(unsafe { (*$GPIOX::ptr()).set_high($i) })
                    }

                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        Ok(unsafe { (*$GPIOX::ptr()).set_low($i) })
                    }
                }

                impl<OUTPUT, AF> StatefulOutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        self.is_set_low().map(|v| !v)
                    }

                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        Ok(unsafe { (*$GPIOX::ptr()).is_set_low($i) })
                    }
                }

                impl<OUTPUT, AF> ToggleableOutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn toggle(&mut self) -> Result<(), Self::Error> {
                        todo!()
                    }
                }

                impl<INPUT, AF> InputPin for $PXi<Input<INPUT>, AF> {

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        self.is_low().map(|v| !v)
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        Ok(unsafe { (*$GPIOX::ptr()).is_low($i) })
                    }
                }

                impl<MODE, AF> ErrorType for $PXi<MODE, AF> {
                    type Error = Infallible;
                }
            )+
        }
    }
}

#[cfg(any(feature = "ht32f1251", feature = "ht32f1252", feature = "ht32f1252"))]
mod ht32f125x;
#[cfg(any(feature = "ht32f1251", feature = "ht32f1252", feature = "ht32f1252"))]
pub use ht32f125x::*;

#[cfg(any(feature = "ht32f1653", feature = "ht32f1654"))]
mod ht32f1653_54;
#[cfg(any(feature = "ht32f1653", feature = "ht32f1654"))]
pub use ht32f1653_54::*;

#[cfg(any(feature = "ht32f1655", feature = "ht32f1656"))]
mod ht32f1655_56;
#[cfg(any(feature = "ht32f1655", feature = "ht32f1656"))]
pub use ht32f1655_56::*;

#[cfg(any(feature = "ht32f1755", feature = "ht32f765"))]
mod ht32f175x;
#[cfg(any(feature = "ht32f1755", feature = "ht32f765"))]
pub use ht32f175x::*;
