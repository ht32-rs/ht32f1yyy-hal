use super::UsbPeripheral;
use core::marker::PhantomData;
use core::slice;
use usb_device::{Result, UsbError};
use vcell::VolatileCell;

pub struct EndpointBuffer<USB> {
    mem: &'static mut [VolatileCell<u32>],
    marker: PhantomData<USB>,
}

pub struct EndpointDoubleBuffer<USB>(pub EndpointBuffer<USB>, pub EndpointBuffer<USB>);

impl<USB: UsbPeripheral> EndpointBuffer<USB> {
    pub fn new(offset_bytes: usize, size_bytes: usize) -> Self {
        let ep_mem_ptr = USB::EP_MEMORY as *mut VolatileCell<u32>;

        let offset_words = offset_bytes >> 2;
        let count_words = size_bytes >> 2;

        unsafe {
            let mem = slice::from_raw_parts_mut(ep_mem_ptr.add(offset_words), count_words);
            Self {
                mem,
                marker: PhantomData,
            }
        }
    }

    #[inline(always)]
    fn read_word(&self, index: usize) -> u32 {
        self.mem[index].get()
    }

    #[inline(always)]
    fn write_word(&self, index: usize, value: u32) {
        self.mem[index].set(value);
    }

    pub fn read(&self, mut buf: &mut [u8]) {
        let mut index = 0;

        while buf.len() >= 4 {
            let word = self.read_word(index);
            buf[0] = (word >> 0) as u8;
            buf[1] = (word >> 8) as u8;
            buf[2] = (word >> 16) as u8;
            buf[3] = (word >> 24) as u8;
            index += 1;

            buf = &mut buf[4..];
        }

        if buf.len() > 0 {
            let word = self.read_word(index);
            buf[0] = ((word >> 0) & 0xff) as u8;
            if buf.len() > 1 {
                buf[1] = ((word >> 8) & 0xff) as u8;
            }
            if buf.len() > 2 {
                buf[2] = ((word >> 16) & 0xff) as u8;
            }
        }
    }

    pub fn write(&self, mut buf: &[u8]) {
        let mut index = 0;

        while buf.len() >= 4 {
            let value: u32 = buf[0] as u32
                | ((buf[1] as u32) << 8)
                | ((buf[2] as u32) << 16)
                | ((buf[3] as u32) << 24);
            self.write_word(index, value);
            index += 1;

            buf = &buf[4..];
        }

        if buf.len() > 0 {
            let mut word = buf[0] as u32;
            if buf.len() > 1 {
                word |= (buf[1] as u32) << 8;
            }
            if buf.len() > 2 {
                word |= (buf[2] as u32) << 16;
            }
            self.write_word(index, word);
        }
    }

    pub fn offset(&self) -> u16 {
        let buffer_address = self.mem.as_ptr() as usize;
        let index = buffer_address - USB::EP_MEMORY as usize;
        index as u16
    }

    pub fn capacity(&self) -> usize {
        self.mem.len() << 2
    }
}

pub struct EndpointMemoryAllocator<USB> {
    next_free_offset: usize,
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> EndpointMemoryAllocator<USB> {
    pub fn new() -> Self {
        Self {
            next_free_offset: 8,
            _marker: PhantomData,
        }
    }

    pub fn setup_buffer() -> EndpointBuffer<USB> {
        EndpointBuffer::new(0, 8)
    }

    pub fn allocate_buffer(&mut self, size: usize) -> Result<EndpointBuffer<USB>> {
        assert_eq!(size & 0b11, 0); // must be 4-byte aligned
        assert!(size < USB::EP_MEMORY_SIZE);

        let offset = self.next_free_offset;
        if offset as usize + size > USB::EP_MEMORY_SIZE {
            return Err(UsbError::EndpointMemoryOverflow);
        }

        self.next_free_offset += size;

        Ok(EndpointBuffer::new(offset, size))
    }

    pub fn allocate_double_buffer(&mut self, size: usize) -> Result<EndpointDoubleBuffer<USB>> {
        assert_eq!(size & 0b11, 0); // must be 4-byte aligned
        assert!(size * 2 < USB::EP_MEMORY_SIZE);

        let offset = self.next_free_offset;
        if offset as usize + 2 * size > USB::EP_MEMORY_SIZE {
            return Err(UsbError::EndpointMemoryOverflow);
        }

        self.next_free_offset += 2 * size;

        Ok(EndpointDoubleBuffer(
            EndpointBuffer::new(offset, size),
            EndpointBuffer::new(offset + size, size),
        ))
    }
}
