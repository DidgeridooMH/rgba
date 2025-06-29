mod bus;
pub use bus::*;

mod interpreter;
pub use interpreter::*;

mod bios;
pub use bios::*;

mod memory;

mod lcd;

use anyhow::{anyhow, Result};
use lcd::Lcd;
use std::{
    fmt,
    sync::{Arc, Mutex},
};

use memory::{system_io::SystemIoFlags, wram::Wram};

#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    OpcodeNotImplemented(u32),
    InvalidRegion(u32),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CoreError::InvalidRegion(address) => {
                write!(f, "Address access violation at 0x{:04X}", address)
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
    pub fn new() -> Self {
        let mut bus = Bus::default();

        bus.register_region(0x4000000..=0x4000056, Arc::new(Mutex::new(Lcd::default())));
        bus.register_region(
            0x4000200..=0x4700000,
            Arc::new(Mutex::new(SystemIoFlags::default())),
        );
        bus.register_region(
            0x3000000..=0x3FFFFFF,
            Arc::new(Mutex::new(Wram::new(0x3000000, 0x8000))),
        );
        bus.register_region(
            0x8000000..=0xFFFFFFF,
            Arc::new(Mutex::new(Wram::new(0x8000000, 0x8000000))),
        );

        let mut cpu = Interpreter::default();
        // TODO: Implement async logging.
        cpu.logging_enabled = true;

        Self { cpu, bus }
    }

    pub fn set_bios(&mut self, bios_path: &str) -> Result<()> {
        let bios = Arc::new(Mutex::new(Bios::new(bios_path)?));
        self.bus.register_region(0..=0x3FFF, bios);
        Ok(())
    }

    pub fn emulate(&mut self, cycles: Option<usize>) -> Result<()> {
        let mut cycles_done = 0;
        loop {
            cycles_done += match self.cpu.tick(&mut self.bus) {
                Ok(cycles) => cycles,
                Err(e) => return Err(anyhow!("{}", e)),
            };

            if let Some(cycles) = cycles {
                if cycles_done >= cycles {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn registers(&self) -> &RegisterBank {
        self.cpu.registers()
    }
}
