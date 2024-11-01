use num_enum::TryFromPrimitive;

use crate::core::interpreter::{
    arm::{DataProcessingInstruction, DataProcessingOperation},
    instruction::{Instruction, Operand},
};

pub const MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_FORMAT: u32 = 0b0010_0000_0000_0000;
pub const MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_MASK: u32 = 0b1110_0000_0000_0000;

pub const ADD_SUBTRACT_FORMAT: u32 = 0b0001_1000_0000_0000;
pub const ADD_SUBTRACT_MASK: u32 = 0b1111_1000_0000_0000;

#[derive(TryFromPrimitive)]
#[repr(u32)]
enum McasOperation {
    Move = 0,
    Compare = 1,
    Add = 2,
    Subtract = 3,
}

pub fn decode_mcas_immediate(opcode: u32) -> Instruction {
    let operation = McasOperation::try_from((opcode >> 11) & 0b11).unwrap();
    let rd = (opcode >> 8) & 0b111;
    let imm8 = Operand::Immediate(opcode & 0xFF);

    match operation {
        McasOperation::Move => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rd,
            imm8,
            Some(rd as u32),
            DataProcessingOperation::Move,
        )),
        McasOperation::Compare => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rd,
            imm8,
            None,
            DataProcessingOperation::Compare,
        )),
        McasOperation::Add => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rd,
            imm8,
            Some(rd as u32),
            DataProcessingOperation::Add,
        )),
        McasOperation::Subtract => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rd,
            imm8,
            Some(rd as u32),
            DataProcessingOperation::Subtract,
        )),
    }
}

pub fn decode_add_subtract(opcode: u32) -> Instruction {
    let operation = (opcode >> 9) & 1 > 0;
    let rd = opcode & 0b111;
    let rs = (opcode >> 3) & 0b11;
    let rn = (opcode >> 6) & 0b111;

    let operand = if (opcode >> 10) & 1 > 0 {
        Operand::Immediate(rn as u32)
    } else {
        Operand::Register(rn)
    };

    Instruction::DataProcessing(DataProcessingInstruction::new(
        true,
        rs,
        operand,
        Some(rd as u32),
        if operation {
            DataProcessingOperation::Subtract
        } else {
            DataProcessingOperation::Add
        },
    ))
}
