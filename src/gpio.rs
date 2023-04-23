//! General Purpose Input / Output

use core::marker::PhantomData;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
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
    fn to_bits(&self) -> u8 {
        match self {
            Self::MA4 => 0b0,
            Self::MA8 => 0b1,
        }
    }
}

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PXx:ident, $pxrst:ident, $pxen:ident, $gpiox_doutr:ident, $gpiox_dinr:ident, $gpiox_drvr:ident, $gpiox_dircr:ident, $gpiox_pur:ident, $gpiox_pdr:ident, $gpiox_iner: ident, $gpiox_odr:ident, [
         $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $AF:ty, $doutx: ident, $dinx: ident, $dirx:ident, $pux: ident, $pdx:ident, $inenx:ident, $odx:ident, $cfgx:ident, $afio_gpxcfgr:ident ),)+
    ]) => {
        pub mod $gpiox {
            use core::convert::Infallible;
            use core::marker::PhantomData;

            use crate::hal::digital::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin, ErrorType};
            use crate::pac::{$GPIOX, RSTCU, AFIO, CKCU};

            use super::{
                Output, Input, OpenDrain, PushPull, PullDown, PullUp, Floating,
                AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7, AF8, AF9, AF10, AF11,
                AF12, AF13, AF14, AF15, GpioExt, Disabled
            };


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
                    let rstcu = unsafe { &*RSTCU::ptr() };
                    let ckcu = unsafe { &*CKCU::ptr() };
                    // reset the GPIO port before using it
                    rstcu.rstcu_apbprstr0.modify(|_, w| w.$pxrst().set_bit());
                    // enable the AHB clock for the GPIO port
                    ckcu.ckcu_apbccr0.modify(|_, w| w.$pxen().set_bit());


                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData, _af: PhantomData },
                        )+
                    }
                }
            }

            /// A general struct that can describe all the pins in this GPIO block,
            /// in case one would have to iterate over them, store them in an array
            /// etc.
            pub struct $PXx<MODE> {
                i: u8,
                _mode: PhantomData<MODE>
            }

            impl<MODE> $PXx<MODE> {
                pub fn get_id(&self) -> u8 {
                    self.i
                }
            }

            // All PXx in any `Output` mode can do this
            impl<OUTPUT> OutputPin for $PXx<Output<OUTPUT>> {
                fn set_high(&mut self) -> Result<(), Self::Error> {
                    // Set the i-th bit of the corresponding GPIO data out register to 1
                    unsafe { (*$GPIOX::ptr()).$gpiox_doutr.modify(|_,w| w.bits(1 << self.i)) };
                    Ok(())
                }

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    // Set the i-th bit of the corresponding GPIO data out register to 0
                    unsafe { (*$GPIOX::ptr()).$gpiox_doutr.modify(|_,w| w.bits(0 << self.i)) };
                    Ok(())
                }
            }

            // All PXx in any `Output` mode can do this
            impl<MODE> StatefulOutputPin for $PXx<Output<MODE>> {
                fn is_set_high(&self) -> Result<bool, Self::Error> {
                    self.is_set_low().map(|v| !v)
                }

                fn is_set_low(&self) -> Result<bool, Self::Error> {
                    // Check whether the i-th bit of the corresponding GPIO data out register is 0
                    Ok(unsafe { (*$GPIOX::ptr()).$gpiox_doutr.read().bits() & (1 << self.i) == 0 })
                }
            }

            // All PXx in any `Output` mode can do this
            impl<MODE> ToggleableOutputPin for $PXx<Output<MODE>> {
                fn toggle(&mut self) -> Result<(), Self::Error> {
                    // TODO
                    Ok(())
                }
            }

            // All PXx in any `Input` mode can do this
            impl<MODE> InputPin for $PXx<Input<MODE>> {
                fn is_high(&self) -> Result<bool, Self::Error> {
                    self.is_low().map(|v| !v)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    // Check whether the i-th bit of the corresponding GPIO data in register is 0
                    Ok(unsafe { (*$GPIOX::ptr()).$gpiox_dinr.read().bits() & (1 << self.i) == 0 })
                }
            }

            impl<MODE> ErrorType for $PXx<MODE> {
                type Error = Infallible;
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
                    /// Change the AF to AF0, leave the IO mode alone though
                    pub fn into_alternate_af0(self) -> $PXi<MODE, AF0> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b00)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF1, leave the IO mode alone though
                    pub fn into_alternate_af1(self) -> $PXi<MODE, AF1> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b01)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF2, leave the IO mode alone though
                    pub fn into_alternate_af2(self) -> $PXi<MODE, AF2> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b10)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF3, leave the IO mode alone though
                    pub fn into_alternate_af3(self) -> $PXi<MODE, AF3> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b11)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

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
                }

                impl<MODE, AF> $PXi<MODE, AF> {
                    /// Change the AF to AF4, leave the IO mode alone though
                    pub fn into_alternate_af4(self) -> $PXi<MODE, AF4> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b0100)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF5, leave the IO mode alone though
                    pub fn into_alternate_af5(self) -> $PXi<MODE, AF5> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b0101)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF6, leave the IO mode alone though
                    pub fn into_alternate_af6(self) -> $PXi<MODE, AF6> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b0110)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF7, leave the IO mode alone though
                    pub fn into_alternate_af7(self) -> $PXi<MODE, AF7> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b0111)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF8, leave the IO mode alone though
                    pub fn into_alternate_af8(self) -> $PXi<MODE, AF8> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1000)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF9, leave the IO mode alone though
                    pub fn into_alternate_af9(self) -> $PXi<MODE, AF9> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1001)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF10, leave the IO mode alone though
                    pub fn into_alternate_af10(self) -> $PXi<MODE, AF10> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1010)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF11, leave the IO mode alone though
                    pub fn into_alternate_af11(self) -> $PXi<MODE, AF11> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1011)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF12, leave the IO mode alone though
                    pub fn into_alternate_af12(self) -> $PXi<MODE, AF12> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1100)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF13, leave the IO mode alone though
                    pub fn into_alternate_af13(self) -> $PXi<MODE, AF13> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1101)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF14, leave the IO mode alone though
                    pub fn into_alternate_af14(self) -> $PXi<MODE, AF14> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1110)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }

                    /// Change the AF to AF15, leave the IO mode alone though
                    pub fn into_alternate_af15(self) -> $PXi<MODE, AF15> {
                        // Enable the AFIO APB clock
                        (unsafe { &*CKCU::ptr() }).ckcu_apbccr0.modify(|_, w| w.afioen().set_bit());
                        // Set the AF
                        unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$cfgx().bits(0b1111)) };

                        $PXi { _mode: PhantomData, _af: PhantomData }
                    }
                }

                impl<MODE, AF> $PXi<MODE, AF> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> $PXx<MODE> {
                        $PXx {
                            i: $i,
                            _mode: self._mode,
                        }
                    }
                }

                impl<OUTPUT, AF> OutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_doutr.modify(|_,w| w.$doutx().set_bit());
                        Ok(())
                    }

                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        (unsafe { &*$GPIOX::ptr() }).$gpiox_doutr.modify(|_,w| w.$doutx().clear_bit());
                        Ok(())
                    }
                }

                impl<OUTPUT, AF> StatefulOutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        self.is_set_low().map(|v| !v)
                    }

                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        Ok((unsafe { &*$GPIOX::ptr() }).$gpiox_doutr.read().$doutx().bit_is_clear())
                    }
                }

                impl<OUTPUT, AF> ToggleableOutputPin for $PXi<Output<OUTPUT>, AF> {
                    fn toggle(&mut self) -> Result<(), Self::Error> {
                        // TODO
                        Ok(())
                    }
                }

                impl<INPUT, AF> InputPin for $PXi<Input<INPUT>, AF> {

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        self.is_low().map(|v| !v)
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        Ok((unsafe { &*$GPIOX::ptr() }).$gpiox_dinr.read().$dinx().bit_is_clear())
                    }
                }

                impl<MODE, AF> ErrorType for $PXi<MODE, AF> {
                    type Error = Infallible;
                }
            )+
        }
    }
}

#[cfg(any(feature = "ht32f175x"))]
gpio!(GPIOA, gpioa, PA, parst, paen, gpioa_doutr, gpioa_dinr, gpioa_drvr, gpioa_dircr, gpioa_pur, gpioa_pdr, gpioa_iner, gpioa_odr, [
    PA0: (pa0, 0, Input<Disabled>, AF0, dout0, din0, dir0, pu0, pd0, inen0, od0, pacfg0, afio_gpacfgr),
    PA1: (pa1, 1, Input<Disabled>, AF0, dout1, din1, dir1, pu1, pd1, inen1, od1, pacfg1, afio_gpacfgr),
    PA2: (pa2, 2, Input<Disabled>, AF0, dout2, din2, dir2, pu2, pd2, inen2, od2, pacfg2, afio_gpacfgr),
    PA3: (pa3, 3, Input<Disabled>, AF0, dout3, din3, dir3, pu3, pd3, inen3, od3, pacfg3, afio_gpacfgr),
    PA4: (pa4, 4, Input<Disabled>, AF0, dout4, din4, dir4, pu4, pd4, inen4, od4, pacfg4, afio_gpacfgr),
    PA5: (pa5, 5, Input<Disabled>, AF0, dout5, din5, dir5, pu5, pd5, inen5, od5, pacfg5, afio_gpacfgr),
    PA6: (pa6, 6, Input<Disabled>, AF0, dout6, din6, dir6, pu6, pd6, inen6, od6, pacfg6, afio_gpacfgr),
    PA7: (pa7, 7, Input<Disabled>, AF0, dout7, din7, dir7, pu7, pd7, inen7, od7, pacfg7, afio_gpacfgr),
    PA8: (pa8, 8, Input<Disabled>, AF0, dout8, din8, dir8, pu8, pd8, inen8, od8, pacfg8, afio_gpacfgr),
    PA9: (pa9, 9, Input<Disabled>, AF0, dout9, din9, dir9, pu9, pd9, inen9, od9, pacfg9, afio_gpacfgr),
    PA10: (pa10, 10, Input<Disabled>, AF0, dout10, din10, dir10, pu10, pd10, inen10, od10, pacfg10, afio_gpacfgr),
    PA11: (pa11, 11, Input<Disabled>, AF0, dout11, din11, dir11, pu11, pd11, inen11, od11, pacfg11, afio_gpacfgr),
    PA12: (pa12, 12, Input<Disabled>, AF0, dout12, din12, dir12, pu12, pd12, inen12, od12, pacfg12, afio_gpacfgr),
    PA13: (pa13, 13, Input<Disabled>, AF0, dout13, din13, dir13, pu13, pd13, inen13, od13, pacfg13, afio_gpacfgr),
    PA14: (pa14, 14, Input<Disabled>, AF0, dout14, din14, dir14, pu14, pd14, inen14, od14, pacfg14, afio_gpacfgr),
    PA15: (pa15, 15, Input<Disabled>, AF0, dout15, din15, dir15, pu15, pd15, inen15, od15, pacfg15, afio_gpacfgr),
]);

#[cfg(any(feature = "ht32f175x"))]
gpio!(GPIOB, gpiob, PB, pbrst, pben, gpiob_doutr, gpiob_dinr, gpiob_drvr, gpiob_dircr, gpiob_pur, gpiob_pdr, gpiob_iner, gpiob_odr, [
    PB0: (pb0, 0, Input<Disabled>, AF0, dout0, din0, dir0, pu0, pd0, inen0, od0, pbcfg0, afio_gpbcfgr),
    PB1: (pb1, 1, Input<Disabled>, AF0, dout1, din1, dir1, pu1, pd1, inen1, od1, pbcfg1, afio_gpbcfgr),
    PB2: (pb2, 2, Input<Disabled>, AF0, dout2, din2, dir2, pu2, pd2, inen2, od2, pbcfg2, afio_gpbcfgr),
    PB3: (pb3, 3, Input<Disabled>, AF0, dout3, din3, dir3, pu3, pd3, inen3, od3, pbcfg3, afio_gpbcfgr),
    PB4: (pb4, 4, Input<Disabled>, AF0, dout4, din4, dir4, pu4, pd4, inen4, od4, pbcfg4, afio_gpbcfgr),
    PB5: (pb5, 5, Input<Disabled>, AF0, dout5, din5, dir5, pu5, pd5, inen5, od5, pbcfg5, afio_gpbcfgr),
    PB6: (pb6, 6, Input<Disabled>, AF0, dout6, din6, dir6, pu6, pd6, inen6, od6, pbcfg6, afio_gpbcfgr),
    PB7: (pb7, 7, Input<Disabled>, AF0, dout7, din7, dir7, pu7, pd7, inen7, od7, pbcfg7, afio_gpbcfgr),
    PB8: (pb8, 8, Input<Disabled>, AF0, dout8, din8, dir8, pu8, pd8, inen8, od8, pbcfg8, afio_gpbcfgr),
    PB9: (pb9, 9, Input<Disabled>, AF0, dout9, din9, dir9, pu9, pd9, inen9, od9, pbcfg9, afio_gpbcfgr),
    PB10: (pb10, 10, Input<Disabled>, AF0, dout10, din10, dir10, pu10, pd10, inen10, od10, pbcfg10, afio_gpbcfgr),
    PB11: (pb11, 11, Input<Disabled>, AF0, dout11, din11, dir11, pu11, pd11, inen11, od11, pbcfg11, afio_gpbcfgr),
    PB12: (pb12, 12, Input<Disabled>, AF0, dout12, din12, dir12, pu12, pd12, inen12, od12, pbcfg12, afio_gpbcfgr),
    PB13: (pb13, 13, Input<Disabled>, AF0, dout13, din13, dir13, pu13, pd13, inen13, od13, pbcfg13, afio_gpbcfgr),
    PB14: (pb14, 14, Input<Disabled>, AF0, dout14, din14, dir14, pu14, pd14, inen14, od14, pbcfg14, afio_gpbcfgr),
    PB15: (pb15, 15, Input<Disabled>, AF0, dout15, din15, dir15, pu15, pd15, inen15, od15, pbcfg15, afio_gpbcfgr),
]);

#[cfg(any(feature = "ht32f175x"))]
gpio!(GPIOC, gpioc, PC, pcrst, pcen, gpioc_doutr, gpioc_dinr, gpioc_drvr, gpioc_dircr, gpioc_pur, gpioc_pdr, gpioc_iner, gpioc_odr, [
    PC0: (pc0, 0, Input<Disabled>, AF0, dout0, din0, dir0, pu0, pd0, inen0, od0, pccfg0, afio_gpccfgr),
    PC1: (pc1, 1, Input<Disabled>, AF0, dout1, din1, dir1, pu1, pd1, inen1, od1, pccfg1, afio_gpccfgr),
    PC2: (pc2, 2, Input<Disabled>, AF0, dout2, din2, dir2, pu2, pd2, inen2, od2, pccfg2, afio_gpccfgr),
    PC3: (pc3, 3, Input<Disabled>, AF0, dout3, din3, dir3, pu3, pd3, inen3, od3, pccfg3, afio_gpccfgr),
    PC4: (pc4, 4, Input<Disabled>, AF0, dout4, din4, dir4, pu4, pd4, inen4, od4, pccfg4, afio_gpccfgr),
    PC5: (pc5, 5, Input<Disabled>, AF0, dout5, din5, dir5, pu5, pd5, inen5, od5, pccfg5, afio_gpccfgr),
    PC6: (pc6, 6, Input<Disabled>, AF0, dout6, din6, dir6, pu6, pd6, inen6, od6, pccfg6, afio_gpccfgr),
    PC7: (pc7, 7, Input<Disabled>, AF0, dout7, din7, dir7, pu7, pd7, inen7, od7, pccfg7, afio_gpccfgr),
    // BOOT0
    PC8: (pc8, 8, Input<PullUp>, AF0, dout8, din8, dir8, pu8, pd8, inen8, od8, pccfg8, afio_gpccfgr),
    // BOOT1
    PC9: (pc9, 9, Input<PullUp>, AF0, dout9, din9, dir9, pu9, pd9, inen9, od9, pccfg9, afio_gpccfgr),
    PC10: (pc10, 10, Input<Disabled>, AF0, dout10, din10, dir10, pu10, pd10, inen10, od10, pccfg10, afio_gpccfgr),
    PC11: (pc11, 11, Input<Disabled>, AF0, dout11, din11, dir11, pu11, pd11, inen11, od11, pccfg11, afio_gpccfgr),
    PC12: (pc12, 12, Input<Disabled>, AF0, dout12, din12, dir12, pu12, pd12, inen12, od12, pccfg12, afio_gpccfgr),
    PC13: (pc13, 13, Input<Disabled>, AF0, dout13, din13, dir13, pu13, pd13, inen13, od13, pccfg13, afio_gpccfgr),
    PC14: (pc14, 14, Input<Disabled>, AF0, dout14, din14, dir14, pu14, pd14, inen14, od14, pccfg14, afio_gpccfgr),
    PC15: (pc15, 15, Input<Disabled>, AF0, dout15, din15, dir15, pu15, pd15, inen15, od15, pccfg15, afio_gpccfgr),
]);

#[cfg(any(feature = "ht32f175x"))]
gpio!(GPIOD, gpiod, PD, pdrst, pden, gpiod_doutr, gpiod_dinr, gpiod_drvr, gpiod_dircr, gpiod_pur, gpiod_pdr, gpiod_iner, gpiod_odr, [
    PD0: (pd0, 0, Input<Disabled>, AF0, dout0, din0, dir0, pu0, pd0, inen0, od0, pdcfg0, afio_gpdcfgr),
    PD1: (pd1, 1, Input<Disabled>, AF0, dout1, din1, dir1, pu1, pd1, inen1, od1, pdcfg1, afio_gpdcfgr),
    PD2: (pd2, 2, Input<Disabled>, AF0, dout2, din2, dir2, pu2, pd2, inen2, od2, pdcfg2, afio_gpdcfgr),
    PD3: (pd3, 3, Input<Disabled>, AF0, dout3, din3, dir3, pu3, pd3, inen3, od3, pdcfg3, afio_gpdcfgr),
    PD4: (pd4, 4, Input<Disabled>, AF0, dout4, din4, dir4, pu4, pd4, inen4, od4, pdcfg4, afio_gpdcfgr),
    PD5: (pd5, 5, Input<Disabled>, AF0, dout5, din5, dir5, pu5, pd5, inen5, od5, pdcfg5, afio_gpdcfgr),
    PD6: (pd6, 6, Input<Disabled>, AF0, dout6, din6, dir6, pu6, pd6, inen6, od6, pdcfg6, afio_gpdcfgr),
    PD7: (pd7, 7, Input<Disabled>, AF0, dout7, din7, dir7, pu7, pd7, inen7, od7, pdcfg7, afio_gpdcfgr),
    PD8: (pd8, 8, Input<Disabled>, AF0, dout8, din8, dir8, pu8, pd8, inen8, od8, pdcfg8, afio_gpdcfgr),
    PD9: (pd9, 9, Input<Disabled>, AF0, dout9, din9, dir9, pu9, pd9, inen9, od9, pdcfg9, afio_gpdcfgr),
    PD10: (pd10, 10, Input<Disabled>, AF0, dout10, din10, dir10, pu10, pd10, inen10, od10, pdcfg10, afio_gpdcfgr),
    PD11: (pd11, 11, Input<Disabled>, AF0, dout11, din11, dir11, pu11, pd11, inen11, od11, pdcfg11, afio_gpdcfgr),
    PD12: (pd12, 12, Input<Disabled>, AF0, dout12, din12, dir12, pu12, pd12, inen12, od12, pdcfg12, afio_gpdcfgr),
    PD13: (pd13, 13, Input<Disabled>, AF0, dout13, din13, dir13, pu13, pd13, inen13, od13, pdcfg13, afio_gpdcfgr),
    PD14: (pd14, 14, Input<Disabled>, AF0, dout14, din14, dir14, pu14, pd14, inen14, od14, pdcfg14, afio_gpdcfgr),
    PD15: (pd15, 15, Input<Disabled>, AF0, dout15, din15, dir15, pu15, pd15, inen15, od15, pdcfg15, afio_gpdcfgr),
]);

#[cfg(any(feature = "ht32f175x"))]
gpio!(GPIOE, gpioe, PD, pdrst, pden, gpioe_doutr, gpioe_dinr, gpioe_drvr, gpioe_dircr, gpioe_pur, gpioe_pdr, gpioe_iner, gpioe_odr, [
    PE0: (pe0, 0, Input<Disabled>, AF0, dout0, din0, dir0, pu0, pd0, inen0, od0, pecfg0, afio_gpecfgr),
    PE1: (pe1, 1, Input<Disabled>, AF0, dout1, din1, dir1, pu1, pd1, inen1, od1, pecfg1, afio_gpecfgr),
    PE2: (pe2, 2, Input<Disabled>, AF0, dout2, din2, dir2, pu2, pd2, inen2, od2, pecfg2, afio_gpecfgr),
    PE3: (pe3, 3, Input<Disabled>, AF0, dout3, din3, dir3, pu3, pd3, inen3, od3, pecfg3, afio_gpecfgr),
    PE4: (pe4, 4, Input<Disabled>, AF0, dout4, din4, dir4, pu4, pd4, inen4, od4, pecfg4, afio_gpecfgr),
    PE5: (pe5, 5, Input<Disabled>, AF0, dout5, din5, dir5, pu5, pd5, inen5, od5, pecfg5, afio_gpecfgr),
    PE6: (pe6, 6, Input<Disabled>, AF0, dout6, din6, dir6, pu6, pd6, inen6, od6, pecfg6, afio_gpecfgr),
    PE7: (pe7, 7, Input<Disabled>, AF0, dout7, din7, dir7, pu7, pd7, inen7, od7, pecfg7, afio_gpecfgr),
    PE8: (pe8, 8, Input<Disabled>, AF0, dout8, din8, dir8, pu8, pd8, inen8, od8, pecfg8, afio_gpecfgr),
    PE9: (pe9, 9, Input<Disabled>, AF0, dout9, din9, dir9, pu9, pd9, inen9, od9, pecfg9, afio_gpecfgr),
    PE10: (pe10, 10, Input<Disabled>, AF0, dout10, din10, dir10, pu10, pd10, inen10, od10, pecfg10, afio_gpecfgr),
    PE11: (pe11, 11, Input<Disabled>, AF0, dout11, din11, dir11, pu11, pd11, inen11, od11, pecfg11, afio_gpecfgr),
    // SWCLK
    PE12: (pe12, 12, Input<PullUp>, AF0, dout12, din12, dir12, pu12, pd12, inen12, od12, pecfg12, afio_gpecfgr),
    // SWDIO
    PE13: (pe13, 13, Input<PullUp>, AF0, dout13, din13, dir13, pu13, pd13, inen13, od13, pecfg13, afio_gpecfgr),
    PE14: (pe14, 14, Input<Disabled>, AF0, dout14, din14, dir14, pu14, pd14, inen14, od14, pecfg14, afio_gpecfgr),
    PE15: (pe15, 15, Input<Disabled>, AF0, dout15, din15, dir15, pu15, pd15, inen15, od15, pecfg15, afio_gpecfgr),
]);
