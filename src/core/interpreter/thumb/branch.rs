use num_enum::TryFromPrimitive;

use crate::core::interpreter::{
    arm::{
        BranchAndExchangeInstruction, BranchInstruction, DataProcessingInstruction,
        DataProcessingOperation,
    },
    instruction::{Instruction, Operand},
};

pub const UNCONDITIONAL_BRANCH_FORMAT: u32 = 0b1110_0000_0000_0000;
pub const UNCONDITIONAL_BRANCH_MASK: u32 = 0b1111_1000_0000_0000;

pub const CONDITIONAL_BRANCH_FORMAT: u32 = 0b1101_0000_0000_0000;
pub const CONDITIONAL_BRANCH_MASK: u32 = 0b1111_0000_0000_0000;

pub const HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_FORMAT: u32 = 0b0100_0100_0000_0000;
pub const HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_MASK: u32 = 0b1111_1100_0000_0000;

pub fn decode_conditional_branch(opcode: u32) -> Instruction {
    let offset = (opcode & 0xFF) as i8;
    Instruction::Branch(BranchInstruction::new(None, (offset as i32) - 4))
}

#[derive(TryFromPrimitive)]
#[repr(u32)]
enum HiRegBxOperation {
    Add = 0,
    Compare = 1,
    Move = 2,
    BranchExchange = 3,
}

pub fn decode_hi_reg_branch_exchange(opcode: u32) -> Instruction {
    let op = HiRegBxOperation::try_from((opcode >> 8) & 0b11).unwrap();
    let rs = (opcode >> 3) & 0b1111;
    let rd = (opcode & 0b111) | ((opcode >> 7) & 1);

    match op {
        HiRegBxOperation::Add => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rs,
            Operand::Register(rd),
            Some(rd as u32),
            DataProcessingOperation::Add,
        )),
        HiRegBxOperation::Compare => Instruction::DataProcessing(DataProcessingInstruction::new(
            true,
            rs,
            Operand::Register(rd),
            None,
            DataProcessingOperation::Compare,
        )),
        HiRegBxOperation::Move => Instruction::DataProcessing(DataProcessingInstruction::new(
            false,
            rs,
            Operand::Register(rd),
            Some(rd as u32),
            DataProcessingOperation::Move,
        )),
        HiRegBxOperation::BranchExchange => {
            Instruction::BranchAndExchange(BranchAndExchangeInstruction::new(rs))
        }
    }
}
