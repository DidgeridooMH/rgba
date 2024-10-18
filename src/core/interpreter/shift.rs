use crate::core::Interpreter;

const SHIFT_TYPE_LSL: u32 = 0;
const SHIFT_TYPE_LSR: u32 = 1;
const SHIFT_TYPE_ASR: u32 = 2;
const SHIFT_TYPE_ROR: u32 = 3;

impl Interpreter {
    pub fn shift_operand(&mut self, opcode: u32) -> u32 {
        let shift_amount = if opcode & (1 << 4) > 0 {
            let shift_register = (opcode >> 8) & 0xF;
            (*self.reg_mut(shift_register as usize)) as u32
        } else {
            ((opcode >> 7) & 0x1F) as u32
        };

        let operand_register_index = opcode & 0xF;
        let operand_register = *self.reg_mut(operand_register_index as usize);
        match (opcode >> 5) & 0b11 {
            SHIFT_TYPE_LSL => operand_register << shift_amount,
            SHIFT_TYPE_LSR => operand_register >> shift_amount,
            SHIFT_TYPE_ASR => ((operand_register as i32) >> shift_amount) as u32,
            SHIFT_TYPE_ROR => operand_register.rotate_right(shift_amount as u32),
            _ => unreachable!(),
        }
    }

    pub fn shift_immediate(opcode: u32) -> u32 {
        let shift_amount = 2 * ((opcode >> 8) & 0xF);
        let immediate = opcode & 0xFF;
        (immediate as u32).rotate_right(shift_amount)
    }
}
