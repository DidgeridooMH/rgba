use crate::core::{Bus, CoreError};

use super::{
    disasm::print_offset_as_immediate, instruction::InstructionExecutor, register::RegisterBank,
};

pub const SOFTWARE_INTERRUPT_MASK: u32 = 0b0000_1111_0000_0000_0000_0000_0000_0000;
pub const SOFTWARE_INTERRUPT_FORMAT: u32 = 0b0000_1111_0000_0000_0000_0000_0000_0000;

const SOFTWARE_INTERRUPT_PC_OFFSET: u32 = 8;

pub struct SoftwareInterruptInstruction {
    past_address: u32,
    comment: u32,
}

impl SoftwareInterruptInstruction {
    pub fn decode(registers: &mut RegisterBank, opcode: u32) -> Self {
        Self {
            past_address: registers.pc(),
            comment: opcode & 0x00FF_FFFF,
        }
    }
}

impl InstructionExecutor for SoftwareInterruptInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        *registers.reg_mut(14) = self.past_address;
        *registers.pc_mut() = SOFTWARE_INTERRUPT_PC_OFFSET;
        *registers.spsr_mut() = registers.cpsr;

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        "swi".into()
    }

    fn description(&self) -> String {
        print_offset_as_immediate(self.comment as i32)
    }
}
