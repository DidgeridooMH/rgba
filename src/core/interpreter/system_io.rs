use crate::core::Addressable;

#[derive(Default)]
pub struct SystemIoFlags {
    post_boot: bool,
}

impl Addressable for SystemIoFlags {
    fn read_byte(&mut self, address: u32) -> u8 {
        match address {
            0x4000300 => self.post_boot as u8,
            _ => {
                println!("Warning: Unhandled read from 0x{:08X}", address);
                0
            }
        }
    }

    fn write_byte(&mut self, address: u32, data: u8) {
        match address {
            0x4000300 => self.post_boot = data > 0,
            _ => {
                println!("Warning: Unhandled write from 0x{:08X}", address);
            }
        }
    }
}
