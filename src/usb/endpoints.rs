use core::marker::PhantomData;
use cortex_m::interrupt::{self, CriticalSection, Mutex};
use usb_device::endpoint::EndpointType;
use usb_device::{Result, UsbDirection, UsbError};

use super::endpoint_memory::{EndpointBuffer, EndpointDoubleBuffer, EndpointMemoryAllocator};
use super::UsbPeripheral;
use crate::pac::usb;

pub struct ControlEndpoint<USB> {
    buf: Option<Mutex<EndpointDoubleBuffer<USB>>>,
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> ControlEndpoint<USB> {
    pub fn new() -> Self {
        Self {
            buf: None,
            _marker: PhantomData,
        }
    }

    pub fn ep_type(&self) -> Option<EndpointType> {
        Some(EndpointType::Control)
    }

    pub fn is_buf_set(&self) -> bool {
        self.buf.is_some()
    }

    /// Set the IN and OUT buffers.
    pub fn set_buf(&mut self, buffer: EndpointDoubleBuffer<USB>) {
        let offset = buffer.0.offset();
        let size = buffer.0.capacity();
        self.buf = Some(Mutex::new(buffer));

        self.regs().cfgr.modify(|_, w| {
            w.epbufa().variant(offset);
            w.eplen().variant(size as u8);
            w.epen().set_bit()
        });
    }

    pub fn configure(&self, cs: &CriticalSection) {
        self.regs().ier.write(|w| {
            w.sdrxie().set_bit();
            w.odrxie().set_bit();
            w.idtxie().set_bit()
        });
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        interrupt::free(|cs| {
            let in_buf = self.buf.as_ref().unwrap().borrow(cs);

            if buf.len() > in_buf.0.capacity() {
                return Err(UsbError::BufferOverflow);
            }

            if self.regs().tcr.read().txcnt().bits() != 0 {
                return Err(UsbError::WouldBlock);
            }

            // EP0: first buffer is for IN
            in_buf.0.write(buf);
            // Set the number of bytes to be transmitted
            self.regs()
                .tcr
                .modify(|_, w| unsafe { w.txcnt().bits(buf.len() as u8) });
            // Clear NAKTX
            if self.regs().csr.read().naktx().bit_is_set() {
                self.regs().csr.write(|w| w.naktx().set_bit());
            }

            Ok(buf.len())
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        interrupt::free(|cs| {
            let istr = self.regs().isr.read();
            let result = if istr.sdrxif().bit_is_set() {
                // Read SETUP packet in dedicated section of EP-SRAM
                let setup_buf = EndpointMemoryAllocator::<USB>::setup_buffer();
                if 8 > buf.len() {
                    Err(UsbError::BufferOverflow)
                } else {
                    setup_buf.read(&mut buf[0..8]);
                    Ok(8)
                }
            } else if istr.odrxif().bit_is_set() {
                let out_buf = self.buf.as_ref().unwrap().borrow(cs);
                let count = self.regs().tcr.read().rxcnt().bits() as usize;
                if count > buf.len() {
                    Err(UsbError::BufferOverflow)
                } else {
                    // EP0: second buffer is for OUT
                    out_buf.1.read(&mut buf[0..count]);
                    Ok(count)
                }
            } else {
                Err(UsbError::WouldBlock)
            };

            // Clear NAKRX
            if self.regs().csr.read().nakrx().bit_is_set() {
                self.regs().csr.write(|w| w.nakrx().set_bit());
            }

            self.regs()
                .isr
                .write(|w| w.sdrxif().set_bit().odrxif().set_bit());
            unsafe { &*crate::pac::USB::ptr() }
                .usb_isr
                .write(|w| w.esofif().set_bit().ep0if().set_bit());

            result
        })
    }

    #[inline(always)]
    fn regs(&self) -> &usb::EP0 {
        let usb = unsafe { &*crate::pac::USB::ptr() };
        &usb.ep0
    }
}

pub struct SingleBufferedEndpoint<USB> {
    buf: Option<Mutex<EndpointBuffer<USB>>>,
    ep_dir: Option<UsbDirection>,
    ep_type: Option<EndpointType>,
    index: u8,
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> SingleBufferedEndpoint<USB> {
    pub fn new(index: u8) -> Self {
        Self {
            buf: None,
            ep_dir: None,
            ep_type: None,
            index,
            _marker: PhantomData,
        }
    }

    pub fn ep_dir(&self) -> Option<UsbDirection> {
        self.ep_dir
    }

    pub fn set_ep_dir(&mut self, ep_dir: UsbDirection) {
        self.ep_dir = Some(ep_dir);
    }

    pub fn ep_type(&self) -> Option<EndpointType> {
        self.ep_type
    }

    pub fn set_ep_type(&mut self, ep_type: EndpointType) {
        self.ep_type = Some(ep_type);
    }

    pub fn is_buf_set(&self) -> bool {
        self.buf.is_some()
    }

    /// Set buffer.
    pub fn set_buf(&mut self, buffer: EndpointBuffer<USB>) {
        let offset = buffer.offset();
        let size = buffer.capacity();
        self.buf = Some(Mutex::new(buffer));

        self.regs().cfgr.modify(|_, w| {
            w.epbufa().variant(offset);
            w.eplen().variant(size as u8)
        });
    }

    pub fn configure(&self, cs: &CriticalSection) {
        let ep_type = match self.ep_type {
            Some(t) => t,
            None => return,
        };

        let ep_dir = match self.ep_dir {
            Some(d) => d,
            None => return,
        };

        self.regs().cfgr.modify(|_, w| {
            w.epadr().variant(self.index + 1);
            w.epdir().bit(ep_dir == UsbDirection::In);
            w.epen().set_bit()
        });

        self.regs().ier.write(|w| {
            w.odrxie().set_bit();
            w.idtxie().set_bit()
        });

        // TODO: move to bus.rs
        let usb = unsafe { &*crate::pac::USB::ptr() };
        usb.usb_ier.modify(|_, w| match self.index {
            0 => w.ep1ie().set_bit(),
            1 => w.ep2ie().set_bit(),
            2 => w.ep3ie().set_bit(),
            _ => w,
        });
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        interrupt::free(|cs| {
            let in_buf = self.buf.as_ref().unwrap().borrow(cs);

            if buf.len() > in_buf.capacity() {
                return Err(UsbError::BufferOverflow);
            }

            if self.regs().tcr.read().tcnt().bits() != 0 {
                return Err(UsbError::WouldBlock);
            }

            in_buf.write(buf);
            self.regs()
                .tcr
                .modify(|_, w| unsafe { w.tcnt().bits(buf.len() as u16) });
            // Clear NAKTX
            if self.regs().csr.read().naktx().bit_is_set() {
                self.regs().csr.write(|w| w.naktx().set_bit());
            }

            Ok(buf.len())
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        interrupt::free(|cs| {
            if self.regs().isr.read().odrxif().bit_is_clear() {
                return Err(UsbError::WouldBlock);
            }

            let out_buf = self.buf.as_ref().unwrap().borrow(cs);

            let count = self.regs().tcr.read().tcnt().bits() as usize;
            if count > buf.len() {
                return Err(UsbError::BufferOverflow);
            }

            out_buf.read(&mut buf[0..count]);

            // Clear NAKRX
            if self.regs().csr.read().nakrx().bit_is_set() {
                self.regs().csr.write(|w| w.nakrx().set_bit());
            }

            self.regs().isr.write(|w| w.odrxif().set_bit());
            unsafe { &*crate::pac::USB::ptr() }.usb_isr.write(|w| {
                w.esofif().set_bit();
                match self.index {
                    0 => w.ep1if().set_bit(),
                    1 => w.ep2if().set_bit(),
                    2 => w.ep3if().set_bit(),
                    _ => unreachable!(),
                }
            });

            Ok(count)
        })
    }

    #[inline(always)]
    fn regs(&self) -> &usb::EPS {
        let usb = unsafe { &*crate::pac::USB::ptr() };
        &usb.eps[self.index as usize]
    }
}

enum MaybeDoubleBuffer<USB> {
    None,
    Single(Mutex<EndpointBuffer<USB>>),
    Double(Mutex<EndpointDoubleBuffer<USB>>),
}

pub struct DoubleBufferedEndpoint<USB> {
    buf: MaybeDoubleBuffer<USB>,
    ep_dir: Option<UsbDirection>,
    ep_type: Option<EndpointType>,
    index: u8,
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> DoubleBufferedEndpoint<USB> {
    pub fn new(index: u8) -> Self {
        Self {
            buf: MaybeDoubleBuffer::None,
            ep_dir: None,
            ep_type: None,
            index,
            _marker: PhantomData,
        }
    }

    pub fn ep_dir(&self) -> Option<UsbDirection> {
        self.ep_dir
    }

    pub fn set_ep_dir(&mut self, ep_dir: UsbDirection) {
        self.ep_dir = Some(ep_dir);
    }

    pub fn ep_type(&self) -> Option<EndpointType> {
        self.ep_type
    }

    pub fn set_ep_type(&mut self, ep_type: EndpointType) {
        self.ep_type = Some(ep_type);
    }

    pub fn is_buf_set(&self) -> bool {
        match self.buf {
            MaybeDoubleBuffer::None => false,
            _ => true,
        }
    }

    /// Set single buffer.
    pub fn set_buf(&mut self, buffer: EndpointBuffer<USB>) {
        let offset = buffer.offset();
        let size = buffer.capacity();
        self.buf = MaybeDoubleBuffer::Single(Mutex::new(buffer));

        self.regs().cfgr.modify(|_, w| {
            w.epbufa().variant(offset);
            w.eplen().variant(size as u16);
            w.sdbs().clear_bit()
        });
    }

    /// Set double buffer.
    pub fn set_double_buf(&mut self, buffer: EndpointDoubleBuffer<USB>) {
        let offset = buffer.0.offset();
        let size = buffer.0.capacity();
        self.buf = MaybeDoubleBuffer::Double(Mutex::new(buffer));

        self.regs().cfgr.modify(|_, w| {
            w.epbufa().variant(offset);
            w.eplen().variant(size as u16);
            w.sdbs().set_bit()
        });
    }

    pub fn configure(&self, cs: &CriticalSection) {
        let ep_type = match self.ep_type {
            Some(t) => t,
            None => return,
        };

        let ep_dir = match self.ep_dir {
            Some(d) => d,
            None => return,
        };

        self.regs().cfgr.modify(|_, w| {
            w.epadr().variant(self.index + 4);
            w.epdir().bit(ep_dir == UsbDirection::In);
            w.eptype().bit(ep_type == EndpointType::Isochronous);
            w.epen().set_bit()
        });

        self.regs().ier.write(|w| {
            w.odrxie().set_bit();
            w.idtxie().set_bit()
        });

        let usb = unsafe { &*crate::pac::USB::ptr() };
        usb.usb_ier.modify(|_, w| match self.index {
            0 => w.ep4ie().set_bit(),
            1 => w.ep5ie().set_bit(),
            2 => w.ep6ie().set_bit(),
            3 => w.ep7ie().set_bit(),
            _ => w,
        });
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        interrupt::free(|cs| {
            let in_buf = match &self.buf {
                MaybeDoubleBuffer::None => return Err(UsbError::InvalidState),
                MaybeDoubleBuffer::Single(s) => s.borrow(cs),
                MaybeDoubleBuffer::Double(d) => todo!(),
            };

            if buf.len() > in_buf.capacity() {
                return Err(UsbError::BufferOverflow);
            }

            if self.regs().tcr.read().tcnt0().bits() != 0 {
                return Err(UsbError::WouldBlock);
            }

            in_buf.write(buf);
            self.regs()
                .tcr
                .modify(|_, w| w.tcnt0().variant(buf.len() as u16));
            self.regs().csr.modify(|_, w| w.naktx().clear_bit());

            Ok(buf.len())
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        interrupt::free(|cs| {
            let out_buf = match &self.buf {
                MaybeDoubleBuffer::None => return Err(UsbError::InvalidState),
                MaybeDoubleBuffer::Single(s) => s.borrow(cs),
                MaybeDoubleBuffer::Double(d) => todo!(),
            };

            let count = self.regs().tcr.read().tcnt0().bits() as usize;
            if count > buf.len() {
                return Err(UsbError::BufferOverflow);
            }

            out_buf.read(&mut buf[0..count]);

            Ok(count)
        })
    }

    #[inline(always)]
    fn regs(&self) -> &usb::EPD {
        let usb = unsafe { &*crate::pac::USB::ptr() };
        &usb.epd[self.index as usize]
    }
}

macro_rules! ep_common {
    ($EP:ident) => {
        impl<USB: UsbPeripheral> $EP<USB> {
            /// Toggle the STALL bit on the endpoint
            pub fn toggle_stalled(&self, dir: UsbDirection) {
                if dir == UsbDirection::Out {
                    self.regs().csr.write(|w| w.stlrx().set_bit());
                } else {
                    self.regs().csr.write(|w| w.stltx().set_bit());
                }
            }

            /// Get stalled status of the endpoint
            pub fn is_stalled(&self, dir: UsbDirection) -> bool {
                if dir == UsbDirection::Out {
                    self.regs().csr.read().stlrx().bit_is_set()
                } else {
                    self.regs().csr.read().stltx().bit_is_set()
                }
            }
        }
    };
}

ep_common!(ControlEndpoint);
ep_common!(SingleBufferedEndpoint);
ep_common!(DoubleBufferedEndpoint);
