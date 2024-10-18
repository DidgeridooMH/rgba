use super::Interpreter;

pub const MULTIPLY_MASK: u32 = 0b0000_1111_1000_0000_0000_0000_1111_0000;
pub const MULTIPLY_FORMAT: u32 = 0b0000_0000_0000_0000_0000_0000_1001_0000;
pub const MULTIPLY_LONG_FORMAT: u32 = 0b0000_0000_1000_0000_0000_0000_1001_0000;

impl Interpreter {
    pub fn multiply(&mut self, opcode: u32) -> usize {
        todo!("Implement multiply")
    }

    pub fn multiply_long(&mut self, opcode: u32) -> usize {
        todo!("Implement multiply_long")
    }
}
