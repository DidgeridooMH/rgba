use crate::core::{Addressable, CoreError};

pub struct Wram {
    start_address: u32,
    container: Vec<u8>,
}

impl Wram {
    pub fn new(start_address: u32, size: usize) -> Self {
        Self {
            start_address,
            container: vec![0; size],
        }
    }

    fn virtual_address(&self, address: u32) -> usize {
        ((address - self.start_address) as usize) % self.container.len()
    }
}

impl Addressable for Wram {
    fn read_byte(&mut self, address: u32) -> u8 {
        self.container[self.virtual_address(address)]
    }

    fn write_byte(&mut self, address: u32, data: u8) -> Result<(), CoreError> {
        let address = self.virtual_address(address);
        self.container[address] = data;
        Ok(())
    }
}
