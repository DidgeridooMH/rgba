use super::{Addressable, CoreError};
use anyhow::{anyhow, Result};
use std::fs;

pub struct Bios([u8; 0x4000]);

impl Bios {
    pub fn new(filename: &str) -> Result<Self> {
        let file = match fs::read(filename) {
            Ok(bios_buffer) => bios_buffer,
            Err(_) => return Err(anyhow!("Unable to find bios file {}", filename)),
        };

        if file.len() != 0x4000 {
            return Err(anyhow!("Bios files must be 0x4000 bytes"));
        }

        Ok(Self(file[0..0x4000].try_into()?))
    }
}

impl Addressable for Bios {
    fn read_byte(&mut self, address: u32) -> u8 {
        self.0[address as usize]
    }

    fn write_byte(&mut self, address: u32, data: u8) -> Result<(), CoreError> {
        println!("BIOS should not be written to. ({address}) <= {data}");
        Err(CoreError::InvalidRegion(address))
    }
}
