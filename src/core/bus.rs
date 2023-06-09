use super::CoreError;
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::rc::Rc;

pub trait Addressable {
    fn read_byte(&mut self, address: u32) -> u8;
    fn write_byte(&mut self, address: u32, data: u8);
}

pub struct MemoryMapping {
    region: RangeInclusive<u32>,
    component: Rc<RefCell<dyn Addressable>>,
}

pub struct Bus {
    regions: Vec<MemoryMapping>,
}

impl Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bus mappings: {{ ")?;
        for r in &self.regions {
            write!(f, "{:?}, ", r.region)?;
        }
        write!(f, " }}")?;

        Ok(())
    }
}

impl Bus {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            regions: Vec::new(),
        }))
    }

    pub fn register_region(
        &mut self,
        region: RangeInclusive<u32>,
        component: Rc<RefCell<dyn Addressable>>,
    ) {
        self.regions.push(MemoryMapping { region, component });
    }

    pub fn read_byte(&mut self, address: u32) -> Result<u8, CoreError> {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                return Ok(mapping.component.borrow_mut().read_byte(address));
            }
        }
        Err(CoreError::InvalidRegion(address))
    }

    pub fn read_word(&mut self, address: u32) -> Result<u32, CoreError> {
        let low_byte = self.read_byte(address)? as u32;
        let high_byte = self.read_byte(address + 1)? as u32;
        Ok(low_byte | (high_byte << 8))
    }

    pub fn read_word_bug(&mut self, address: u32) -> Result<u16, CoreError> {
        let low_byte = self.read_byte(address)? as u16;
        let high_byte = self.read_byte((address & 0xFF00) | ((address + 1) & 0xFF))? as u16;
        Ok(low_byte | (high_byte << 8))
    }

    pub fn write_byte(&mut self, address: u32, data: u8) -> Result<(), CoreError> {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                mapping.component.borrow_mut().write_byte(address, data);
                return Ok(());
            }
        }
        Err(CoreError::InvalidRegion(address))
    }

    pub fn write_word(&mut self, address: u32, data: u16) -> Result<(), CoreError> {
        self.write_byte(address, data as u8)?;
        self.write_byte(address + 1, (data >> 8) as u8)?;
        Ok(())
    }
}
