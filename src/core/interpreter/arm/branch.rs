use crate::core::{Bus, CoreError};

use super::disasm::print_offset_as_immediate;
use super::instruction::InstructionExecutor;
use super::register::RegisterBank;
use super::status::InstructionMode;

pub const BRANCH_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BRANCH_FORMAT: u32 = 0b0000_1010_0000_0000_0000_0000_0000_0000;

pub const BRANCH_AND_EXCHANGE_MASK: u32 = 0b0000_1111_1111_1111_1111_1111_1111_0000;
pub const BRANCH_AND_EXCHANGE_FORMAT: u32 = 0b0000_0001_0010_1111_1111_1111_0001_0000;

const BRANCH_CYCLE_COUNT: usize = 3;

pub struct BranchInstruction {
    link: bool,
    offset: i32,
}

impl BranchInstruction {
    pub fn decode(_registers: &mut RegisterBank, opcode: u32) -> Self {
        Self {
            link: opcode & (1 << 24) > 0,
            offset: ((opcode & 0x00FF_FFFF) << 10) as i32 >> 8,
        }
    }
}

impl InstructionExecutor for BranchInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        if self.link {
            *registers.reg_mut(14) = registers.pc();
        }
        *registers.pc_mut() = (registers.pc() as i32 + self.offset as i32) as u32;
        Ok(BRANCH_CYCLE_COUNT)
    }

    fn mnemonic(&self) -> String {
        if self.link { "bl" } else { "b" }.into()
    }

    fn description(&self) -> String {
        print_offset_as_immediate(self.offset)
    }
}

pub struct BranchAndExchangeInstruction {
    pub target_register: u32,
}

impl BranchAndExchangeInstruction {
    pub fn decode(_registers: &mut RegisterBank, opcode: u32) -> Self {
        Self {
            target_register: opcode & 0xF,
        }
    }
}

impl InstructionExecutor for BranchAndExchangeInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        let target_address = registers.reg(self.target_register as usize);
        *registers.pc_mut() = target_address & !1;
        registers.cpsr.instruction_mode = if target_address & 1 > 0 {
            InstructionMode::Thumb
        } else {
            InstructionMode::Arm
        };

        Ok(BRANCH_CYCLE_COUNT)
    }

    fn mnemonic(&self) -> String {
        "bx".into()
    }

    fn description(&self) -> String {
        format!("r{}", self.target_register)
    }
}
