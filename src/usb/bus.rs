//! USB peripheral driver.

use core::mem::{self, MaybeUninit};
use cortex_m::interrupt::{self, Mutex};
use usb_device::bus::{PollResult, UsbBusAllocator};
use usb_device::endpoint::{EndpointAddress, EndpointType};
use usb_device::{Result, UsbDirection, UsbError};

use super::endpoint_memory::EndpointMemoryAllocator;
use super::endpoints::{ControlEndpoint, DoubleBufferedEndpoint, SingleBufferedEndpoint};
use super::registers::UsbRegisters;
use super::UsbPeripheral;

/// USB peripheral driver for STM32 microcontrollers.
pub struct UsbBus<USB> {
    peripheral: USB,
    regs: Mutex<UsbRegisters<USB>>,
    epc: ControlEndpoint<USB>,             // EP0
    eps: [SingleBufferedEndpoint<USB>; 3], // EP1-3
    epd: [DoubleBufferedEndpoint<USB>; 4], // EP4-7
    ep_allocator: EndpointMemoryAllocator<USB>,
}

impl<USB: UsbPeripheral> UsbBus<USB> {
    /// Constructs a new USB peripheral driver.
    pub fn new(peripheral: USB) -> UsbBusAllocator<Self> {
        USB::enable(&peripheral);

        let bus = UsbBus {
            peripheral,
            regs: Mutex::new(UsbRegisters::new()),
            ep_allocator: EndpointMemoryAllocator::new(),
            epc: ControlEndpoint::new(),
            eps: {
                let mut endpoints: [MaybeUninit<SingleBufferedEndpoint<USB>>; 3] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for i in 0..3 {
                    endpoints[i] = MaybeUninit::new(SingleBufferedEndpoint::new(i as u8));
                }

                unsafe { mem::transmute::<_, [SingleBufferedEndpoint<USB>; 3]>(endpoints) }
            },
            epd: {
                let mut endpoints: [MaybeUninit<DoubleBufferedEndpoint<USB>>; 4] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for i in 0..4 {
                    endpoints[i] = MaybeUninit::new(DoubleBufferedEndpoint::new(i as u8));
                }

                unsafe { mem::transmute::<_, [DoubleBufferedEndpoint<USB>; 4]>(endpoints) }
            },
        };

        UsbBusAllocator::new(bus)
    }

    pub fn free(self) -> USB {
        self.peripheral
    }

    /// Simulates a disconnect from the USB bus, causing the host to reset and re-enumerate the
    /// device.
    ///
    /// Mostly used for development. By calling this at the start of your program ensures that the
    /// host re-enumerates your device after a new program has been flashed.
    pub fn force_reenumeration(&self) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            let pdwn = regs.usb_csr.read().pdwn().bit_is_set();
            regs.usb_csr.modify(|_, w| w.pdwn().set_bit());
            regs.usb_csr.modify(|_, w| w.pdwn().bit(pdwn));
        });
    }
}

impl<USB: UsbPeripheral> usb_device::bus::UsbBus for UsbBus<USB> {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        _interval: u8,
    ) -> Result<EndpointAddress> {
        match ep_type {
            EndpointType::Control => {
                if let Some(a) = ep_addr {
                    if a.index() != 0 {
                        // Only EP0 can be used as control endpoint
                        return Err(UsbError::InvalidEndpoint);
                    }
                }
                if !self.epc.is_buf_set() {
                    let buffer = self
                        .ep_allocator
                        .allocate_double_buffer(max_packet_size as usize)?;
                    self.epc.set_buf(buffer);
                }

                return Ok(EndpointAddress::from_parts(0, ep_dir));
            }
            EndpointType::Interrupt | EndpointType::Bulk => {
                let range = if let Some(a) = ep_addr {
                    if a.index() == 0 {
                        // EP0 is always a control endpoint
                        return Err(UsbError::InvalidEndpoint);
                    }
                    if a.index() < 4 {
                        a.index() - 1..a.index()
                    } else {
                        // EP4-7 need to be handled slightly differently
                        todo!();
                    }
                } else {
                    0..3
                };

                // TODO: should also try EP4-7 if EP1-3 are already used
                for index in range {
                    let ep = &mut self.eps[index];

                    match ep.ep_type() {
                        None => {
                            ep.set_ep_type(ep_type);
                        }
                        Some(t) if t != ep_type => {
                            continue;
                        }
                        _ => {}
                    };

                    match ep.ep_dir() {
                        None => {
                            ep.set_ep_dir(ep_dir);
                        }
                        Some(d) if d != ep_dir => {
                            continue;
                        }
                        _ => {}
                    }

                    if ep.is_buf_set() {
                        continue;
                    }

                    let buffer = self
                        .ep_allocator
                        .allocate_buffer(max_packet_size as usize)?;

                    ep.set_buf(buffer);

                    return Ok(EndpointAddress::from_parts(index + 1, ep_dir));
                }
            }
            EndpointType::Isochronous => todo!(),
        };

        Err(match ep_addr {
            Some(_) => UsbError::InvalidEndpoint,
            None => UsbError::EndpointOverflow,
        })
    }

    fn enable(&mut self) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            regs.usb_csr.write(|w| {
                #[cfg(feature = "dppu")]
                w.dppuen().set_bit().dpwken().set_bit();
                w.pdwn().set_bit().lpmode().set_bit()
            });

            #[cfg(not(feature = "dppu"))]
            self.peripheral.dp_pull_up();

            regs.usb_isr.write(|w| unsafe { w.bits(!0) });

            #[cfg(feature = "dppu")]
            regs.usb_csr.modify(|_, w| w.dpwken().clear_bit());

            regs.usb_ier.write(|w| {
                w.ugie().set_bit();
                w.sofie().set_bit();
                w.urstie().set_bit();
                w.rsmie().set_bit();
                w.suspie().set_bit();
                w.ep0ie().set_bit()
            });
        });
    }

    fn reset(&self) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            regs.usb_csr.modify(|r, w| {
                unsafe { w.bits(0) };
                #[cfg(feature = "dppu")]
                if r.dppuen().bit_is_set() {
                    w.dppuen().set_bit();
                }
                w
            });

            regs.usb_ier.write(|w| {
                w.ugie().set_bit();
                w.sofie().set_bit();
                w.urstie().set_bit();
                w.rsmie().set_bit();
                w.suspie().set_bit();
                w.ep0ie().set_bit()
            });

            self.epc.configure(cs);

            for ep in self.eps.iter() {
                ep.configure(cs);
            }

            for ep in self.epd.iter() {
                ep.configure(cs);
            }
        });
    }

    fn set_device_address(&self, addr: u8) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);
            regs.usb_devar
                .modify(|_, w| w.deva().variant(addr as u8));
        });
    }

    fn poll(&self) -> PollResult {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);
            let istr = regs.usb_isr.read();

            if istr.rsmif().bit_is_set() {
                regs.usb_isr
                    .write(|w| w.esofif().set_bit().rsmif().set_bit());
                PollResult::Resume
            } else if istr.urstif().bit_is_set() {
                regs.usb_isr
                    .write(|w| w.esofif().set_bit().urstif().set_bit());
                PollResult::Reset
            } else if istr.suspif().bit_is_set() {
                regs.usb_isr
                    .write(|w| w.esofif().set_bit().suspif().set_bit());
                PollResult::Suspend
            } else if istr.bits() & 0xFF00 != 0 {
                let mut ep_out = 0;
                let mut ep_in_complete = 0;
                let mut ep_setup = 0;

                // XXX: temporary structure to get things working
                if istr.ep0if().bit_is_set() {
                    let ep_istr = regs.ep0.isr.read();
                    if ep_istr.sdrxif().bit_is_set() {
                        ep_setup |= 1 << 0;
                    }
                    if ep_istr.odrxif().bit_is_set() {
                        ep_out |= 1 << 0;
                    }
                    if ep_istr.idtxif().bit_is_set() {
                        ep_in_complete |= 1 << 0;
                        regs.ep0.isr.write(|w| w.idtxif().set_bit());
                    }
                    regs.usb_isr
                        .write(|w| w.esofif().set_bit().ep0if().set_bit());
                }
                if istr.ep1if().bit_is_set() {
                    let ep_istr = regs.eps[0].isr.read();
                    if ep_istr.odrxif().bit_is_set() {
                        ep_out |= 1 << 1;
                    }
                    if ep_istr.idtxif().bit_is_set() {
                        ep_in_complete |= 1 << 1;
                        regs.eps[0].isr.write(|w| w.idtxif().set_bit());
                    }
                    regs.usb_isr
                        .write(|w| w.esofif().set_bit().ep1if().set_bit());
                }
                if istr.ep2if().bit_is_set() {
                    let ep_istr = regs.eps[1].isr.read();
                    if ep_istr.odrxif().bit_is_set() {
                        ep_out |= 1 << 2;
                    }
                    if ep_istr.idtxif().bit_is_set() {
                        ep_in_complete |= 1 << 2;
                        regs.eps[1].isr.write(|w| w.idtxif().set_bit());
                    }
                    regs.usb_isr
                        .write(|w| w.esofif().set_bit().ep2if().set_bit());
                }
                if istr.ep3if().bit_is_set() {
                    let ep_istr = regs.eps[2].isr.read();
                    if ep_istr.odrxif().bit_is_set() {
                        ep_out |= 1 << 3;
                    }
                    if ep_istr.idtxif().bit_is_set() {
                        ep_in_complete |= 1 << 3;
                        regs.eps[2].isr.write(|w| w.idtxif().set_bit());
                    }
                    regs.usb_isr
                        .write(|w| w.esofif().set_bit().ep3if().set_bit());
                }
                // TODO: EP4-7

                PollResult::Data {
                    ep_out,
                    ep_in_complete,
                    ep_setup,
                }
            } else {
                PollResult::None
            }
        })
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> Result<usize> {
        if !ep_addr.is_in() {
            return Err(UsbError::InvalidEndpoint);
        }

        match ep_addr.index() {
            0 => self.epc.write(buf),
            1..=3 => self.eps[ep_addr.index() - 1].write(buf),
            4..=7 => self.epd[ep_addr.index() - 4].write(buf),
            _ => Err(UsbError::InvalidEndpoint),
        }
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> Result<usize> {
        if !ep_addr.is_out() {
            return Err(UsbError::InvalidEndpoint);
        }

        match ep_addr.index() {
            0 => self.epc.read(buf),
            1..=3 => self.eps[ep_addr.index() - 1].read(buf),
            4..=7 => self.epd[ep_addr.index() - 4].read(buf),
            _ => Err(UsbError::InvalidEndpoint),
        }
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        interrupt::free(|_| {
            if self.is_stalled(ep_addr) == stalled {
                return;
            }

            match ep_addr.index() {
                0 => self.epc.toggle_stalled(ep_addr.direction()),
                1..=3 => self.eps[ep_addr.index() - 1].toggle_stalled(ep_addr.direction()),
                4..=7 => self.epd[ep_addr.index() - 4].toggle_stalled(ep_addr.direction()),
                _ => {}
            }
        });
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        match ep_addr.index() {
            0 => self.epc.is_stalled(ep_addr.direction()),
            1..=3 => self.eps[ep_addr.index() - 1].is_stalled(ep_addr.direction()),
            4..=7 => self.epd[ep_addr.index() - 4].is_stalled(ep_addr.direction()),
            _ => panic!("Invalid endpoint address"),
        }
    }

    fn suspend(&self) {
        interrupt::free(|cs| {
            self.regs
                .borrow(cs)
                .usb_csr
                .modify(|_, w| w.lpmode().set_bit());
        });
    }

    fn resume(&self) {
        interrupt::free(|cs| {
            self.regs
                .borrow(cs)
                .usb_csr
                .modify(|_, w| w.lpmode().clear_bit());
        });
    }
}

