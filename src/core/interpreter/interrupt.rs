use super::{CpuMode, Interpreter};

pub const SOFTWARE_INTERRUPT_MASK: u32 = 0b0000_1111_0000_0000_0000_0000_0000_0000;
pub const SOFTWARE_INTERRUPT_FORMAT: u32 = 0b0000_1111_0000_0000_0000_0000_0000_0000;

const SOFTWARE_INTERRUPT_PC_OFFSET: u32 = 8;

impl Interpreter {
    pub fn software_interrupt(&mut self, opcode: u32) -> usize {
        *self.reg_with_mode_mut(14, CpuMode::Supervisor) = self.pc();
        *self.pc_mut() = SOFTWARE_INTERRUPT_PC_OFFSET;
        *self.spsr_with_mode_mut(CpuMode::Supervisor) = self.cpsr;

        self.log_instruction(opcode, "SWI", &format!("0x{:X}", opcode & 0x00FFFFFF));

        1
    }
}
