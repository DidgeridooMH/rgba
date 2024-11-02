use crate::core::interpreter::{
    arm::SingleDataTransferInstruction,
    instruction::{Instruction, Operand},
};

pub const LOAD_STORE_WITH_REGISTER_OFFSET_FORMAT: u32 = 0b0101_0000_0000_0000;
pub const LOAD_STORE_WITH_REGISTER_OFFSET_MASK: u32 = 0b1111_0010_0000_0000;

pub const PC_RELATIVE_LOAD_FORMAT: u32 = 0b0100_1000_0000_0000;
pub const PC_RELATIVE_LOAD_MASK: u32 = 0b1111_1000_0000_0000;

pub const SP_RELATIVE_LOAD_STORE_FORMAT: u32 = 0b1001_0000_0000_0000;
pub const SP_RELATIVE_LOAD_STORE_MASK: u32 = 0b1111_0000_0000_0000;

pub fn decode_load_store_register_offset(opcode: u32) -> Instruction {
    let load = (opcode >> 11) & 1 > 0;
    let byte = (opcode >> 10) & 1 > 0;
    let ro = (opcode >> 6) & 0b111;
    let rb = (opcode >> 3) & 0b111;
    let rd = opcode & 0b111;

    Instruction::SingleDataTransfer(SingleDataTransferInstruction::new(
        rd,
        rb,
        Operand::Register(ro),
        load,
        false,
        byte,
        true,
        true,
        false,
    ))
}

pub fn decode_pc_relative_load(opcode: u32) -> Instruction {
    let rd = (opcode >> 8) & 0b111;
    let word8 = (opcode & 0xFF) << 2;
    Instruction::SingleDataTransfer(SingleDataTransferInstruction::new(
        rd,
        15,
        Operand::Immediate(word8 as u32),
        true,
        false,
        false,
        true,
        true,
        true,
    ))
}

pub fn decode_sp_relative_load_store(opcode: u32) -> Instruction {
    let load = (opcode >> 11) & 1 > 0;
    let rd = (opcode >> 8) & 0b111;
    let word8 = (opcode & 0xFF) << 2;
    Instruction::SingleDataTransfer(SingleDataTransferInstruction::new(
        rd,
        13,
        Operand::Immediate(word8 as u32),
        load,
        false,
        false,
        true,
        true,
        false,
    ))
}
