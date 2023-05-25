//! Clock Control Unit + Reset Control Unit
use crate::pac::{CKCU, FMC, RSTCU};
use crate::time::{Hertz, RateExtU32};

/// Extension trait that constrains the `Ckcu` peripheral
pub trait CkcuExt {
    /// Constrains the `Ckcu` peripheral so it plays nicely with the other abstractions
    fn constrain(self, rstcu: RSTCU) -> Ckcu;
}

impl CkcuExt for CKCU {
    // Also take RSTCU here so it is impossible to safely generate resets for
    // peripherals
    fn constrain(self, _rstcu: RSTCU) -> Ckcu {
        Ckcu {
            configuration: Configuration {
                ckout: None,
                hse: None,
                lse: None,
                ck_usb: None,
                ck_adc_ip: None,
                hclk: None,
                ck_sys: None,
            },
        }
    }
}

/// Constrained Ckcu peripheral
pub struct Ckcu {
    pub configuration: Configuration,
}

/// High Speed Internal Oscillator at 8 Mhz
const HSI: u32 = 8_000_000;
/// Low Speed Internal Oscillator at 32 Khz
const LSI: u32 = 32_000;

/// All clocks that can be outputted via CKOUT.
/// See User Manual page 91.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CkoutSrc {
    /// Output the CK_REF, no prescaler
    CkRef,
    /// Output the HCLK, divided by 16
    Hclk,
    /// Output the CK_SYS, divided by 16
    CkSys,
    /// Output the CK_HSE, divided by 16
    CkHse,
    /// Output the CK_HSI, divided by 16
    CkHsi,
    /// Output the CK_LSE, no prescaler
    CkLse,
    /// Output the CK_LSI, no prescaler
    CkLsi,
}

/// Representation of the HT32F52342 clock tree.
///
/// Note that this struct only represents the targeted values.
/// As there are constrains as to which clock values can be achieved,
/// these values will probably never be achieved to 100% correctness.
/// See User Manual page 83
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Configuration {
    /// Which clock should be outputted via CKOUT
    ckout: Option<CkoutSrc>,
    /// The frequency of an HSE, should one be given
    hse: Option<Hertz>,
    /// The frequency of an LSI, should one be given.
    lse: Option<Hertz>,
    /// The optimal frequency for CK_USB, aka the USB clock
    ck_usb: Option<Hertz>,
    /// The optimal frequency for CK_ADC_IP, aka the ADC clock
    ck_adc_ip: Option<Hertz>,
    /// The optimal frequency for CK_SYS
    ck_sys: Option<Hertz>,
    /// The optimal frequency for HCLK, aka the AHB bus
    hclk: Option<Hertz>,
}

/// Frozen core clock frequencies
///
/// The existence of this value indicates that the core clock
/// configuration can no longer be changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clocks {
    /// Which clock should be outputted via CKOUT, if any
    pub(crate) ckout: Option<CkoutSrc>,
    /// The frequency for CK_USB, aka the USB clock
    pub(crate) ck_usb: Hertz,
    /// The frequency for CK_ADC_IP, aka the ADC clock
    pub(crate) ck_adc_ip: Hertz,
    /// The frequency for CK_SYS
    pub(crate) ck_sys: Hertz,
    /// The frequency for STCLK, aka the SysTick clock
    pub(crate) stclk: Hertz,
    /// The frequency for HCLK, aka the AHB bus
    pub(crate) hclk: Hertz,
}

impl Configuration {
    /// Set the clock that should be outputted via CKOUT
    pub fn ckout(mut self, ckout: CkoutSrc) -> Self {
        self.ckout = Some(ckout);
        self
    }

    /// Notifies the Configuration mechanism that an HSE is in use, this
    /// will make it prefer the HSE over the HSI in case the HSI should
    /// turn out to be the fitting clock for a certain part of the
    /// configuration.
    pub fn use_hse<F>(mut self, hse: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hse = Some(hse.into());
        self
    }

    /// Notifies the Configuration mechanism that an LSE is in use, this
    /// will make it prefer the LSE over the LSI in case the LSI should
    /// turn out to be the fitting clock for a certain part of the
    /// configuration.
    pub fn use_lse<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.lse = Some(freq.into());
        self
    }

    /// Sets the desired value for CK_USB
    pub fn ck_usb<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.ck_usb = Some(freq.into());
        self
    }

    /// Sets the desired value for CK_ADC_IP
    pub fn ck_adc_ip<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.ck_adc_ip = Some(freq.into());
        self
    }

    /// Sets the desired value for CK_SYS
    pub fn ck_sys<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.ck_sys = Some(freq.into());
        self
    }

    /// Sets the desired value for HCLK
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into());
        self
    }

    /// Freeze the configuration into a Clocks struct and apply it
    pub fn freeze(self) -> Clocks {
        // High speed oscillator
        let hso = self.hse.unwrap_or_else(|| HSI.Hz());
        // PLL source clock, see top left corner of the clock tree
        let pllsrc = self.hse.is_none();

        let mut pll_target_clock = None;

        // Refer to User manual for SW values
        let (sw, mut ck_sys) = match self.ck_sys {
            Some(ck_sys) => {
                // Maximum frequency for CK_SYS is 144 Mhz
                // Refer to CKCU Block Diagram in User Manual
                assert!(ck_sys <= 144.MHz::<1, 1>());

                if self.hse.map(|h| h == ck_sys).unwrap_or(false) {
                    (0b10, self.hse.unwrap())
                } else if ck_sys.raw() == HSI {
                    (0b11, HSI.Hz())
                }
                // If no exact match is found, use the pll
                else {
                    pll_target_clock = Some(ck_sys);
                    (0b00, ck_sys)
                }
            }
            // If no value is given select the high speed oscillator,
            // furthermore automatically choose HSE if it's provided.
            None => match self.hse {
                Some(hse) => (0b10, hse),
                None => (0b11, HSI.Hz()),
            },
        };

        match self.ck_usb {
            Some(ck_usb) => {
                // Maximum frequency for CK_USB is 48 Mhz
                // Refer to CKCU Block Diagram in User Manual
                assert!(ck_usb <= 48.MHz::<1, 1>());
                if pll_target_clock.is_none() {
                    pll_target_clock = self.ck_usb;
                }
                ck_usb
            }
            None => 0.Hz(),
        };

        let (mut nf2, mut no2) = (None, None);
        if let Some(pll_target) = pll_target_clock {
            // According to User Manual: pll_out = CK_in (NF2/NO2)
            let optimal_divider = pll_target.raw() as f32 / hso.raw() as f32;
            let mut closest = (1, 1);
            let mut difference = f32::MAX;

            // Try all combinations of NF2 and NO2, there are only
            // 256 so this should be fine.
            for nf2 in 1..=64 {
                // VCO_out = CK_in * NF2
                // and VCO_out must be between 64 and 144 Mhz
                let vco_out = hso.raw() * nf2;
                if (64_000_000..=144_000_000).contains(&vco_out) {
                    for no2 in &[1, 2, 4, 8] {
                        let current_divider = nf2 as f32 / *no2 as f32;

                        // The maximum output frequency for the PLL must be
                        // between 8 and 144 Mhz
                        let current_output = current_divider * hso.raw() as f32;
                        if !(8_000_000.0..=144_000_000.0).contains(&current_output) {
                            continue;
                        }

                        let mut current_difference = optimal_divider - current_divider;
                        if current_difference < 0.0 {
                            current_difference *= -1.0
                        }

                        if current_difference < difference {
                            closest = (nf2 as u8, *no2);
                            difference = current_difference;
                        }
                    }
                }
            }

            let ck_pll = ((hso.raw() as f32 * (closest.0 as f32 / closest.1 as f32)) as u32).Hz();
            if sw == 0b00 {
                ck_sys = ck_pll;
            }

            // Map NF2 values to their respective register values
            closest.0 = if closest.0 == 64 { 0 } else { closest.0 };

            // Map NO2 values to their respective register values
            // Refer to User manual page 88
            closest.1 = match closest.1 {
                1 => 0b00,
                2 => 0b01,
                4 => 0b10,
                8 => 0b11,
                _ => unreachable!(),
            };

            nf2 = Some(closest.0);
            no2 = Some(closest.1);
        }

        // Calculate the AHB clock prescaler
        // hclk = ck_sys / ahb prescaler
        let (ahb_div, hclk) = match self.hclk {
            Some(hclk) => {
                let (bits, div) = match ck_sys.raw() / hclk.raw() {
                    0 => unreachable!(),
                    1 => (0b00, 1),
                    2..=3 => (0b01, 2),
                    4..=7 => (0b10, 4),
                    _ => (0b11, 8),
                };

                (bits, (ck_sys.raw() / div).Hz())
            }
            None => (0b000, ck_sys),
        };

        let (usb_div, ck_usb) = match self.ck_usb {
            Some(usbclk) => {
                // TODO: this should be ck_pll, since there is no guarantuee ck_sys == ck_pll
                let (bits, div) = match ck_sys.raw() / usbclk.raw() {
                    0 => unreachable!(),
                    1 => (0b00, 1),
                    2 => (0b01, 2),
                    _ => (0b10, 3),
                };
                (bits, (ck_sys.raw() / div).Hz())
            }
            None => (0b10, 0.Hz()),
        };

        // SysTick clock
        let stclk = (hclk.raw() / 8).Hz();

        // Calculate the ADC clock prescaler
        // ck_adc_ip = hclk / adc prescaler
        let (adc_div, ck_adc_ip) = match self.ck_adc_ip {
            Some(ck_adc_ip) => {
                let (bits, div) = match hclk.raw() / ck_adc_ip.raw() {
                    0 => unreachable!(),
                    1 => (0b000, 1),
                    2..=3 => (0b001, 2),
                    4..=5 => (0b010, 4),
                    6..=7 => (0b111, 6),
                    8..=15 => (0b011, 8),
                    16..=31 => (0b100, 16),
                    32..=63 => (0b101, 32),
                    _ => (0b110, 64),
                };

                (bits, (hclk.raw() / div).Hz())
            }
            None => (0b000, hclk),
        };

        // Apply the calculated clock configuration
        let ckcu = unsafe { &*CKCU::ptr() };

        // Enable backup domain, necessary for USB.
        // TODO: only do this if ck_usb is Some?
        ckcu.ckcu_lpcr.write(|w| w.bkiso().set_bit());

        // First configure the PLL in case it needs to be set up
        if pll_target_clock.is_some() {
            // Set the source clock for the PLL
            ckcu.ckcu_gcfgr.modify(|_, w| w.pllsrc().bit(pllsrc));

            // Set the actual configuration values
            ckcu.ckcu_pllcfgr.modify(|_, w| unsafe {
                w.pfbd() // PFBD contains NF2
                 .bits(nf2.unwrap())
                 .potd() // POTD contains NO2
                 .bits(no2.unwrap())
            });

            // Enable the PLL
            ckcu.ckcu_gccr.modify(|_, w| w.pllen().set_bit());

            // Wait for the PLL to become stable
            while !ckcu.ckcu_gcsr.read().pllrdy().bit_is_set() {
                cortex_m::asm::nop();
            }
        }

        // Set the flash wait states so the chip doesn't hang on higher frequencies
        let fmc = unsafe { &*FMC::ptr() };
        if hclk > 48.MHz::<1, 1>() {
            fmc.fmc_cfcr.modify(|_, w| unsafe { w.wait().bits(0b011) });
        } else if hclk > 24.MHz::<1, 1>() {
            fmc.fmc_cfcr.modify(|_, w| unsafe { w.wait().bits(0b010) });
        }

        // Set up the proper CK_SYS source
        ckcu.ckcu_gccr.modify(|_, w| unsafe { w.sw().bits(sw) });

        // Set the AHB prescaler
        ckcu.ckcu_ahbcfgr
            .modify(|_, w| unsafe { w.ahbpre().bits(ahb_div) });

        // Set the USB prescaler
        ckcu.ckcu_gcfgr
            .modify(|_, w| unsafe { w.usbpre().bits(usb_div) });

        // Set the ADC prescaler
        ckcu.ckcu_apbcfgr
            .modify(|_, w| unsafe { w.adcdiv().bits(adc_div) });

        // After all clocks are set up, configure CKOUT if required
        if let Some(ckout) = self.ckout {
            let ckout = match ckout {
                CkoutSrc::CkRef => 0b000,
                CkoutSrc::Hclk => 0b001,
                CkoutSrc::CkSys => 0b010,
                CkoutSrc::CkHse => 0b011,
                CkoutSrc::CkHsi => 0b100,
                CkoutSrc::CkLse => 0b101,
                CkoutSrc::CkLsi => 0b110,
            };

            ckcu.ckcu_gcfgr
                .modify(|_, w| unsafe { w.ckoutsrc().bits(ckout) });
        }

        // Reset AFIO here because the GPIO implementation is block wise ->
        // Resetting AFIO during GPIO initialization could lead to already being
        // used pins / their AF being reset.
        (unsafe { &*RSTCU::ptr() })
            .rstcu_apbprstr0
            .modify(|_, w| w.afiorst().set_bit());

        Clocks {
            ckout: self.ckout,
            ck_usb,
            ck_adc_ip,
            ck_sys,
            stclk,
            hclk,
        }
    }
}

/// Peripheral Clock Enable and Reset
pub(crate) trait Pcer {
    fn enable(&self);
    fn disable(&self);
    fn reset(&self);
}

macro_rules! pcer {
    (
        $(($PERI:ident, $bccr:ident, $bccb:ident, $brstr:ident, $brstb:ident),)+
    ) => {
        $(
            impl Pcer for crate::pac::$PERI {
                fn enable(&self) {
                    let ckcu = unsafe { &*CKCU::ptr() };
                    ckcu.$bccr.modify(|_, w| w.$bccb().set_bit());
                }

                fn disable(&self) {
                    let ckcu = unsafe { &*CKCU::ptr() };
                    ckcu.$bccr.modify(|_, w| w.$bccb().clear_bit());
                }

                fn reset(&self) {
                    let rstcu = unsafe { &*RSTCU::ptr() };
                    rstcu.$brstr.modify(|_, w| w.$brstb().set_bit());
                }
            }
        )+
    }
}

// COMMON
pcer!(
    (I2C0,  ckcu_apbccr0, i2c0en,  rstcu_apbprstr0, i2c0rst),
    (I2C1,  ckcu_apbccr0, i2c1en,  rstcu_apbprstr0, i2c1rst),
    (SPI0,  ckcu_apbccr0, spi0en,  rstcu_apbprstr0, spi0rst),
    (SPI1,  ckcu_apbccr0, spi1en,  rstcu_apbprstr0, spi1rst),
    (AFIO,  ckcu_apbccr0, afioen,  rstcu_apbprstr0, afiorst),
    (EXTI,  ckcu_apbccr0, extien,  rstcu_apbprstr0, extirst),
    (SCI,   ckcu_apbccr0, scien,   rstcu_apbprstr0, scirst),
    (GPTM0, ckcu_apbccr1, gptm0en, rstcu_apbprstr1, gptm0rst),
    (GPTM1, ckcu_apbccr1, gptm1en, rstcu_apbprstr1, gptm1rst),
    (BFTM0, ckcu_apbccr1, bftm0en, rstcu_apbprstr1, bftm0rst),
    (BFTM1, ckcu_apbccr1, bftm1en, rstcu_apbprstr1, bftm1rst),
    (ADC,   ckcu_apbccr1, adcen,   rstcu_apbprstr1, adcrst),
);

// UART
#[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
pcer!(
    (USART0, ckcu_apbccr0, ur0en, rstcu_apbprstr0, ur0rst),
    (USART1, ckcu_apbccr0, ur1en, rstcu_apbprstr0, ur1rst),
);

#[cfg(any(
    feature = "ht32f1653",
    feature = "ht32f1654",
    feature = "ht32f1655",
    feature = "ht32f1656",
))]
pcer!(
    (UART0,  ckcu_apbccr0, ur0en,  rstcu_apbprstr0, ur0rst),
    (UART1,  ckcu_apbccr0, ur1en,  rstcu_apbprstr0, ur1rst),
    (USART0, ckcu_apbccr0, usr0en, rstcu_apbprstr0, usr0rst),
    (USART1, ckcu_apbccr0, usr1en, rstcu_apbprstr0, usr1rst),
);

// GPIO
#[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
pcer!(
    (GPIOA, ckcu_apbccr0, paen, rstcu_apbprstr0, parst),
    (GPIOB, ckcu_apbccr0, pben, rstcu_apbprstr0, pbrst),
    (GPIOC, ckcu_apbccr0, pcen, rstcu_apbprstr0, pcrst),
    (GPIOD, ckcu_apbccr0, pden, rstcu_apbprstr0, pdrst),
    (GPIOE, ckcu_apbccr0, peen, rstcu_apbprstr0, perst),
);

#[cfg(any(
    feature = "ht32f1653",
    feature = "ht32f1654",
    feature = "ht32f1655",
    feature = "ht32f1656",
))]
pcer!(
    (GPIOA, ckcu_ahbccr, paen, rstcu_ahbprstr, parst),
    (GPIOB, ckcu_ahbccr, pben, rstcu_ahbprstr, pbrst),
    (GPIOC, ckcu_ahbccr, pcen, rstcu_ahbprstr, pcrst),
    (GPIOD, ckcu_ahbccr, pden, rstcu_ahbprstr, pdrst),
);

#[cfg(any(feature = "ht32f1655", feature = "ht32f1656"))]
pcer!(
    (GPIOE, ckcu_ahbccr, peen, rstcu_ahbprstr, perst),
);

// USB
#[cfg(any(feature = "ht32f1755", feature = "ht32f1765"))]
pcer!(
    (USB, ckcu_apbccr1, usben, rstcu_apbprstr1, usbrst),
);

#[cfg(any(
    feature = "ht32f1653",
    feature = "ht32f1654",
    feature = "ht32f1655",
    feature = "ht32f1656",
))]
pcer!(
    (USB, ckcu_ahbccr, usben, rstcu_ahbprstr, usbrst),
);

// CRC
#[cfg(any(
    feature = "ht32f1653",
    feature = "ht32f1654",
    feature = "ht32f1655",
    feature = "ht32f1656",
))]
pcer!(
    (CRC, ckcu_ahbccr, crcen, rstcu_ahbprstr, crcrst),
);
