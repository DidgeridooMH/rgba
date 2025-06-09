use super::{Addressable, CoreError};

#[derive(Default)]
pub struct Lcd {}

impl Addressable for Lcd {
    fn read_byte(&mut self, _address: u32) -> u8 {
        0
    }

    fn write_byte(&mut self, _address: u32, _data: u8) -> Result<(), CoreError> {
        Ok(())
    }
}
