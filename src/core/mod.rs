mod bus;
pub use bus::*;

mod cpu;
pub use cpu::*;

mod bios;
pub use bios::*;

use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    AddressDecode(u8),
    OpcodeNotImplemented(u8),
    InvalidRegion(u32),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CoreError::InvalidRegion(address) => {
                write!(f, "Address access violation at 0x{:04X}", address)
            }
            CoreError::AddressDecode(opcode) => {
                write!(f, "Unknown address mode from 0x{:02X}", opcode)
            }
            CoreError::OpcodeNotImplemented(opcode) => {
                write!(f, "Opcode not implemented: 0x{0:02X}", opcode)
            }
        }
    }
}

pub struct Gba {
    cycle_count: usize,
    cpu: Cpu,
}

impl Gba {
    pub fn new(bios_filename: &str) -> Result<Self, String> {
        let bus = Bus::new();

        bus.borrow_mut()
            .register_region(0..=0x3FFF, Rc::new(RefCell::new(Bios::new(bios_filename))));

        Ok(Self {
            cycle_count: 0,
            cpu: Cpu::default(),
        })
    }

    pub fn emulate(&mut self, cycles: usize) -> Result<(), String> {
        Ok(())
    }
}
