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
use std::{cell::RefCell, fmt, rc::Rc, time::Instant};

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
    pub fn new(bios_filename: &str) -> Result<Self> {
        let mut bus = Bus::default();

        let bios = Bios::new(bios_filename)?;
        bus.register_region(0..=0x3FFF, Rc::new(RefCell::new(bios)));
        bus.register_region(0x4000000..=0x4000056, Rc::new(RefCell::new(Lcd::default())));
        bus.register_region(
            0x4000200..=0x4700000,
            Rc::new(RefCell::new(SystemIoFlags::default())),
        );
        bus.register_region(
            0x3000000..=0x3FFFFFF,
            Rc::new(RefCell::new(Wram::new(0x3000000, 0x8000))),
        );
        bus.register_region(
            0x8000000..=0xFFFFFFF,
            Rc::new(RefCell::new(Wram::new(0x8000000, 0x8000000))),
        );

        let mut cpu = Interpreter::default();
        // TODO: Implement async logging.
        cpu.logging_enabled = true;

        Ok(Self {
            cpu,
            bus,
        })
    }

    pub fn emulate(&mut self, cycles: Option<usize>) -> Result<()> {
        let start = Instant::now();
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
            }
        }
        let elapsed = start.elapsed();
        let speed =  cycles_done as f64 / elapsed.as_secs_f64();

        println!("Cycles completed: {cycles_done}");
        println!("Elapsed time: {}ms", elapsed.as_millis());
        println!(
            "Instructions per second: {speed}",
        );

        const NECESSARY_SPEED: f64 = (16.78 * 1e6) / 4.0;
        if speed < NECESSARY_SPEED {
            println!(
                "Warning: Emulation speed is too slow. Speed: {speed:.0} Instructions per second, Necessary speed: {NECESSARY_SPEED:.0} Instructions per second"
            );
        }


        Ok(())
    }
}
