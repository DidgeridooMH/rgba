use num_enum::TryFromPrimitive;

use crate::core::{
    interpreter::{
        arm::{
            BranchAndExchangeInstruction, BranchInstruction, DataProcessingInstruction,
            DataProcessingOperation,
        },
        instruction::{Instruction, InstructionExecutor, Operand},
        register::RegisterBank,
    },
    CoreError,
};

pub const UNCONDITIONAL_BRANCH_FORMAT: u32 = 0b1110_0000_0000_0000;
pub const UNCONDITIONAL_BRANCH_MASK: u32 = 0b1111_1000_0000_0000;

pub const CONDITIONAL_BRANCH_FORMAT: u32 = 0b1101_0000_0000_0000;
pub const CONDITIONAL_BRANCH_MASK: u32 = 0b1111_0000_0000_0000;

pub const HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_FORMAT: u32 = 0b0100_0100_0000_0000;
pub const HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_MASK: u32 = 0b1111_1100_0000_0000;

pub const LONG_BRANCH_WITH_LINK_FORMAT: u32 = 0b1111_0000_0000_0000;
pub const LONG_BRANCH_WITH_LINK_MASK: u32 = 0b1111_0000_0000_0000;

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

pub struct LongBranchWithLinkInstruction {
    offset: u32,
    h: bool,
}

impl LongBranchWithLinkInstruction {
    pub fn decode(opcode: u32) -> Self {
        Self {
            offset: opcode & 0x7FF,
            h: (opcode >> 11) & 1 > 0,
        }
    }
}

impl InstructionExecutor for LongBranchWithLinkInstruction {
    fn execute(
        &self,
        registers: &mut RegisterBank,
        _bus: &mut crate::core::Bus,
    ) -> Result<usize, CoreError> {
        if self.h {
            let address = (registers.reg(14) + (self.offset << 1)) & !1;
            let pc = registers.pc() - 2;
            *registers.pc_mut() = address;
            *registers.reg_mut(14) = pc | 1;
        } else {
            let address = registers
                .pc()
                .wrapping_add((self.offset << 12) | 0xFF800000);
            *registers.reg_mut(14) = address;
        }

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        if self.h {
            "blh".into()
        } else {
            "bl".into()
        }
    }

    fn description(&self, registers: &RegisterBank, _bus: &mut crate::core::Bus) -> String {
        let address_hint = if self.h {
            registers.reg(14) + (self.offset << 1)
        } else {
            registers.pc() + (self.offset << 12)
        };

        format!("#{:X} (=${:08X})", self.offset, address_hint)
    }
}
