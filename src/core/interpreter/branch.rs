use super::{InstructionMode, Interpreter};

pub const BRANCH_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BRANCH_FORMAT: u32 = 0b0000_1010_0000_0000_0000_0000_0000_0000;

pub const BRANCH_AND_EXCHANGE_MASK: u32 = 0b0000_1111_1111_1111_1111_1111_1111_0000;
pub const BRANCH_AND_EXCHANGE_FORMAT: u32 = 0b0000_0001_0010_1111_1111_1111_0001_0000;

const BRANCH_CYCLE_COUNT: usize = 3;

impl Interpreter {
    /// Branches to a new address potentially linking the old address to register 14.
    ///
    /// # Arguments
    ///
    /// * `opcode` - The opcode to interpret.
    ///
    /// # Returns
    ///
    /// The number of cycles taken to execute the instruction.
    pub fn branch(&mut self, opcode: u32) -> usize {
        // Offsets are in groups of 4s and signed extended 24 bit.
        let offset = ((opcode & 0x00FF_FFFF) << 10) as i32 >> 8;
        let new_pc = self.pc() as i32 + offset as i32 + 4;

        let mneumonic = if opcode & (1 << 24) > 0 { "B" } else { "BL" };
        self.log_instruction(
            opcode,
            mneumonic,
            &format!("(0x{offset:X}) -> ${new_pc:08X}"),
        );

        // Save the old PC address to the link register.
        if opcode & (1 << 24) > 0 {
            *self.reg_mut(14) = self.pc();
        }

        *self.pc_mut() = new_pc as u32;
        BRANCH_CYCLE_COUNT
    }

    /// Branches to a new address and switches the instruction mode.
    ///
    /// # Arguments
    ///
    /// * `opcode` - The opcode to interpret.
    ///
    /// # Returns
    ///
    /// The number of cycles taken to execute the instruction.
    pub fn branch_and_exchange(&mut self, opcode: u32) -> usize {
        let target_register = (opcode & 0xF) as usize;
        let target_address = self.reg(target_register);

        let new_pc = target_address & !1;
        let new_mode = if target_address & 1 > 0 {
            InstructionMode::Thumb
        } else {
            InstructionMode::Arm
        };

        self.log_instruction(opcode, "BX", &format!("{target_register} -> ${new_pc:08X}"));

        *self.pc_mut() = new_pc;
        self.instruction_mode = new_mode;

        BRANCH_CYCLE_COUNT
    }
}
