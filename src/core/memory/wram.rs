use crate::core::Addressable;

pub struct Wram {
    start_address: u32,
    container: Vec<u8>
}

impl Wram {
    pub fn new(start_address: u32, size: usize) -> Self {
        Self {
            start_address,
            container: vec![0; size]
        }
    }
}

impl Addressable for Wram {
    fn read_byte(&mut self, address: u32) -> u8 {
        self.container[(address - self.start_address) as usize]
    }

    fn write_byte(&mut self, address: u32, data: u8) {
        self.container[(address - self.start_address) as usize] = data;
    }
}
