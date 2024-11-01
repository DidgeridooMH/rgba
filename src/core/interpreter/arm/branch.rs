use crate::core::{Bus, CoreError};

use crate::core::interpreter::disasm::print_offset_as_immediate;
use crate::core::interpreter::instruction::InstructionExecutor;
use crate::core::interpreter::register::RegisterBank;
use crate::core::interpreter::status::InstructionMode;

pub const BRANCH_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BRANCH_FORMAT: u32 = 0b0000_1010_0000_0000_0000_0000_0000_0000;

pub const BRANCH_AND_EXCHANGE_MASK: u32 = 0b0000_1111_1111_1111_1111_1111_1111_0000;
pub const BRANCH_AND_EXCHANGE_FORMAT: u32 = 0b0000_0001_0010_1111_1111_1111_0001_0000;

const BRANCH_CYCLE_COUNT: usize = 3;

pub struct BranchInstruction {
    link: Option<u32>,
    offset: i32,
}

impl BranchInstruction {
    pub fn new(link: Option<u32>, offset: i32) -> Self {
        Self { link, offset }
    }

    pub fn decode(registers: &mut RegisterBank, opcode: u32) -> Self {
        Self {
            link: if opcode & (1 << 24) > 0 {
                Some(registers.pc())
            } else {
                None
            },
            offset: ((opcode & 0x00FF_FFFF) << 10) as i32 >> 8,
        }
    }
}

impl InstructionExecutor for BranchInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        if let Some(link) = self.link {
            *registers.reg_mut(14) = link;
        }
        *registers.pc_mut() = (registers.pc() as i32 + self.offset as i32) as u32;
        Ok(BRANCH_CYCLE_COUNT)
    }

    fn mnemonic(&self) -> String {
        if let Some(_) = self.link { "bl" } else { "b" }.into()
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        print_offset_as_immediate(self.offset)
    }
}

pub struct BranchAndExchangeInstruction {
    pub target_register: u32,
}

impl BranchAndExchangeInstruction {
    pub fn new(target_register: u32) -> Self {
        Self { target_register }
    }

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

    fn description(&self, registers: &RegisterBank, _bus: &mut Bus) -> String {
        format!(
            "r{} (=${:X})",
            self.target_register,
            registers.reg(self.target_register as usize)
        )
    }
}
