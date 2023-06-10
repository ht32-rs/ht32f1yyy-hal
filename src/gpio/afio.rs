//! Alternate Function I/O Control Unit
use crate::pac::AFIO;
use crate::ckcu::Pcer;

pub struct Afio {
    inner: AFIO,
}

impl Afio {
    pub fn new(afio: AFIO) -> Self {
        // AFIO clock enable
        afio.enable();

        Self { inner: afio }
    }

    pub fn release(self) -> AFIO {
        self.inner
    }
}

pub(crate) trait AfioCfg {
    fn into_alternate(&self, afio: &mut Afio, af: u8);
}

macro_rules! afio {
    ($gpiox:ident: [
         $($PXi:ident: ($afio_gpxcfgr:ident, $pxcfgi:ident),)+
    ]) => {
        use crate::gpio::$gpiox::*;
        $(
            impl<MODE, AF> AfioCfg for $PXi<MODE, AF> {
                fn into_alternate(&self, _afio: &mut Afio, af: u8) {
                    // NOTE (Safety): we already have an exclusive reference to AFIO
                    unsafe { (*AFIO::ptr()).$afio_gpxcfgr.modify(|_, w| w.$pxcfgi().bits(af)) };
                }
            }
        )+
    }
}

#[cfg(feature = "afio4")]
afio!(gpioa: [
    PA0: (afio_gpacfgr, pacfg0),
    PA1: (afio_gpacfgr, pacfg1),
    PA2: (afio_gpacfgr, pacfg2),
    PA3: (afio_gpacfgr, pacfg3),
    PA4: (afio_gpacfgr, pacfg4),
    PA5: (afio_gpacfgr, pacfg5),
    PA6: (afio_gpacfgr, pacfg6),
    PA7: (afio_gpacfgr, pacfg7),
    PA8: (afio_gpacfgr, pacfg8),
    PA9: (afio_gpacfgr, pacfg9),
    PA10: (afio_gpacfgr, pacfg10),
    PA11: (afio_gpacfgr, pacfg11),
    PA12: (afio_gpacfgr, pacfg12),
    PA13: (afio_gpacfgr, pacfg13),
    PA14: (afio_gpacfgr, pacfg14),
    PA15: (afio_gpacfgr, pacfg15),
]);

#[cfg(not(feature = "afio4"))]
afio!(gpioa: [
    PA0: (afio_gpacfglr, pacfg0),
    PA1: (afio_gpacfglr, pacfg1),
    PA2: (afio_gpacfglr, pacfg2),
    PA3: (afio_gpacfglr, pacfg3),
    PA4: (afio_gpacfglr, pacfg4),
    PA5: (afio_gpacfglr, pacfg5),
    PA6: (afio_gpacfglr, pacfg6),
    PA7: (afio_gpacfglr, pacfg7),
    PA8: (afio_gpacfghr, pacfg8),
    PA9: (afio_gpacfghr, pacfg9),
    PA10: (afio_gpacfghr, pacfg10),
    PA11: (afio_gpacfghr, pacfg11),
    PA12: (afio_gpacfghr, pacfg12),
    PA13: (afio_gpacfghr, pacfg13),
    PA14: (afio_gpacfghr, pacfg14),
    PA15: (afio_gpacfghr, pacfg15),
]);

#[cfg(feature = "afio4")]
afio!(gpiob: [
    PB0: (afio_gpbcfgr, pbcfg0),
    PB1: (afio_gpbcfgr, pbcfg1),
    PB2: (afio_gpbcfgr, pbcfg2),
    PB3: (afio_gpbcfgr, pbcfg3),
    PB4: (afio_gpbcfgr, pbcfg4),
    PB5: (afio_gpbcfgr, pbcfg5),
    PB6: (afio_gpbcfgr, pbcfg6),
    PB7: (afio_gpbcfgr, pbcfg7),
    PB8: (afio_gpbcfgr, pbcfg8),
    PB9: (afio_gpbcfgr, pbcfg9),
    PB10: (afio_gpbcfgr, pbcfg10),
    PB11: (afio_gpbcfgr, pbcfg11),
    PB12: (afio_gpbcfgr, pbcfg12),
    PB13: (afio_gpbcfgr, pbcfg13),
    PB14: (afio_gpbcfgr, pbcfg14),
    PB15: (afio_gpbcfgr, pbcfg15),
]);

#[cfg(not(feature = "afio4"))]
afio!(gpiob: [
    PB0: (afio_gpbcfglr, pbcfg0),
    PB1: (afio_gpbcfglr, pbcfg1),
    PB2: (afio_gpbcfglr, pbcfg2),
    PB3: (afio_gpbcfglr, pbcfg3),
    PB4: (afio_gpbcfglr, pbcfg4),
    PB5: (afio_gpbcfglr, pbcfg5),
    PB6: (afio_gpbcfglr, pbcfg6),
    PB7: (afio_gpbcfglr, pbcfg7),
    PB8: (afio_gpbcfghr, pbcfg8),
    PB9: (afio_gpbcfghr, pbcfg9),
    PB10: (afio_gpbcfghr, pbcfg10),
    PB11: (afio_gpbcfghr, pbcfg11),
    PB12: (afio_gpbcfghr, pbcfg12),
    PB13: (afio_gpbcfghr, pbcfg13),
    PB14: (afio_gpbcfghr, pbcfg14),
    PB15: (afio_gpbcfghr, pbcfg15),
]);

#[cfg(feature = "afio4")]
afio!(gpioc: [
    PC0: (afio_gpccfgr, pccfg0),
    PC1: (afio_gpccfgr, pccfg1),
    PC2: (afio_gpccfgr, pccfg2),
    PC3: (afio_gpccfgr, pccfg3),
    PC4: (afio_gpccfgr, pccfg4),
    PC5: (afio_gpccfgr, pccfg5),
    PC6: (afio_gpccfgr, pccfg6),
    PC7: (afio_gpccfgr, pccfg7),
    PC8: (afio_gpccfgr, pccfg8),
    PC9: (afio_gpccfgr, pccfg9),
    PC10: (afio_gpccfgr, pccfg10),
    PC11: (afio_gpccfgr, pccfg11),
    PC12: (afio_gpccfgr, pccfg12),
    PC13: (afio_gpccfgr, pccfg13),
    PC14: (afio_gpccfgr, pccfg14),
    PC15: (afio_gpccfgr, pccfg15),
]);

#[cfg(not(feature = "afio4"))]
afio!(gpioc: [
    PC0: (afio_gpccfglr, pccfg0),
    PC1: (afio_gpccfglr, pccfg1),
    PC2: (afio_gpccfglr, pccfg2),
    PC3: (afio_gpccfglr, pccfg3),
    PC4: (afio_gpccfglr, pccfg4),
    PC5: (afio_gpccfglr, pccfg5),
    PC6: (afio_gpccfglr, pccfg6),
    PC7: (afio_gpccfglr, pccfg7),
    PC8: (afio_gpccfghr, pccfg8),
    PC9: (afio_gpccfghr, pccfg9),
    PC10: (afio_gpccfghr, pccfg10),
    PC11: (afio_gpccfghr, pccfg11),
    PC12: (afio_gpccfghr, pccfg12),
    PC13: (afio_gpccfghr, pccfg13),
    PC14: (afio_gpccfghr, pccfg14),
    PC15: (afio_gpccfghr, pccfg15),
]);

#[cfg(feature = "afio4")]
afio!(gpiod: [
    PD0: (afio_gpdcfgr, pdcfg0),
    PD1: (afio_gpdcfgr, pdcfg1),
    PD2: (afio_gpdcfgr, pdcfg2),
    PD3: (afio_gpdcfgr, pdcfg3),
    PD4: (afio_gpdcfgr, pdcfg4),
    PD5: (afio_gpdcfgr, pdcfg5),
    PD6: (afio_gpdcfgr, pdcfg6),
    PD7: (afio_gpdcfgr, pdcfg7),
    PD8: (afio_gpdcfgr, pdcfg8),
    PD9: (afio_gpdcfgr, pdcfg9),
    PD10: (afio_gpdcfgr, pdcfg10),
    PD11: (afio_gpdcfgr, pdcfg11),
    PD12: (afio_gpdcfgr, pdcfg12),
    PD13: (afio_gpdcfgr, pdcfg13),
    PD14: (afio_gpdcfgr, pdcfg14),
    PD15: (afio_gpdcfgr, pdcfg15),
]);

#[cfg(not(any(feature = "afio4", feature = "ht32f1653", feature = "ht32f1654")))]
afio!(gpiod: [
    PD0: (afio_gpdcfglr, pdcfg0),
    PD1: (afio_gpdcfglr, pdcfg1),
    PD2: (afio_gpdcfglr, pdcfg2),
    PD3: (afio_gpdcfglr, pdcfg3),
    PD4: (afio_gpdcfglr, pdcfg4),
    PD5: (afio_gpdcfglr, pdcfg5),
    PD6: (afio_gpdcfglr, pdcfg6),
    PD7: (afio_gpdcfglr, pdcfg7),
    PD8: (afio_gpdcfghr, pdcfg8),
    PD9: (afio_gpdcfghr, pdcfg9),
    PD10: (afio_gpdcfghr, pdcfg10),
    PD11: (afio_gpdcfghr, pdcfg11),
    PD12: (afio_gpdcfghr, pdcfg12),
    PD13: (afio_gpdcfghr, pdcfg13),
    PD14: (afio_gpdcfghr, pdcfg14),
    PD15: (afio_gpdcfghr, pdcfg15),
]);

#[cfg(any(feature = "ht32f1653", feature = "ht32f1654"))]
afio!(gpiod: [
    PD0: (afio_gpdcfglr, pdcfg0),
    PD1: (afio_gpdcfglr, pdcfg1),
    PD2: (afio_gpdcfglr, pdcfg2),
]);

#[cfg(feature = "afio4")]
afio!(gpioe: [
    PE0: (afio_gpecfgr, pecfg0),
    PE1: (afio_gpecfgr, pecfg1),
    PE2: (afio_gpecfgr, pecfg2),
    PE3: (afio_gpecfgr, pecfg3),
    PE4: (afio_gpecfgr, pecfg4),
    PE5: (afio_gpecfgr, pecfg5),
    PE6: (afio_gpecfgr, pecfg6),
    PE7: (afio_gpecfgr, pecfg7),
    PE8: (afio_gpecfgr, pecfg8),
    PE9: (afio_gpecfgr, pecfg9),
    PE10: (afio_gpecfgr, pecfg10),
    PE11: (afio_gpecfgr, pecfg11),
    PE12: (afio_gpecfgr, pecfg12),
    PE13: (afio_gpecfgr, pecfg13),
    PE14: (afio_gpecfgr, pecfg14),
    PE15: (afio_gpecfgr, pecfg15),
]);

#[cfg(not(any(feature = "afio4", feature = "ht32f1653", feature = "ht32f1654")))]
afio!(gpioe: [
    PE0: (afio_gpecfglr, pecfg0),
    PE1: (afio_gpecfglr, pecfg1),
    PE2: (afio_gpecfglr, pecfg2),
    PE3: (afio_gpecfglr, pecfg3),
    PE4: (afio_gpecfglr, pecfg4),
    PE5: (afio_gpecfglr, pecfg5),
    PE6: (afio_gpecfglr, pecfg6),
    PE7: (afio_gpecfglr, pecfg7),
    PE8: (afio_gpecfghr, pecfg8),
    PE9: (afio_gpecfghr, pecfg9),
    PE10: (afio_gpecfghr, pecfg10),
    PE11: (afio_gpecfghr, pecfg11),
    PE12: (afio_gpecfghr, pecfg12),
    PE13: (afio_gpecfghr, pecfg13),
    PE14: (afio_gpecfghr, pecfg14),
    PE15: (afio_gpecfghr, pecfg15),
]);
