use super::Addressable;
use std::fs;

pub struct Bios([u8; 0x4000]);

impl Bios {
    pub fn new(filename: &str) -> Self {
        let file = fs::read(filename).unwrap();
        Self {
            0: file[0..0x4000].try_into().unwrap(),
        }
    }
}

impl Addressable for Bios {
    fn read_byte(&mut self, address: u32) -> u8 {
        self.0[address as usize]
    }

    fn write_byte(&mut self, address: u32, data: u8) {
        unimplemented!("BIOS should not be written to. ({address}) <= {data}")
    }
}
