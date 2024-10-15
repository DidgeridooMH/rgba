mod bus;
pub use bus::*;

mod interpreter;
pub use interpreter::*;

mod bios;
pub use bios::*;

use anyhow::{anyhow, Result};
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    AddressDecode(u8),
    OpcodeNotImplemented(u32),
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
                write!(f, "Opcode not implemented: 0x{0:08X}", opcode)
            }
        }
    }
}

pub struct Gba {
    cpu: Interpreter,
    bus: Bus,
}

impl Gba {
    pub fn new(bios_filename: &str) -> Result<Self> {
        let mut bus = Bus::default();

        let bios = Bios::new(bios_filename)?;
        bus.register_region(0..=0x3FFF, Rc::new(RefCell::new(bios)));

        Ok(Self {
            cpu: Interpreter::default(),
            bus,
        })
    }

    pub fn emulate(&mut self, cycles: usize) -> Result<()> {
        let mut cycles_done = 0;
        while cycles_done < cycles {
            cycles_done += match self.cpu.tick(&mut self.bus) {
                Ok(cycles) => cycles,
                Err(e) => return Err(anyhow!("{}", e)),
            };
        }

        Ok(())
    }
}
