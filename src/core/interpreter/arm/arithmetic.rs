use crate::core::{Bus, CoreError};

use crate::core::interpreter::{
    instruction::{InstructionExecutor, Operand},
    register::RegisterBank,
    shift::{rotated_immediate, Shift},
};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

pub const DATA_PROCESSING_MASK: u32 = 0x0C000000;
pub const DATA_PROCESSING_FORMAT: u32 = 0x00000000;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum DataProcessingOperation {
    // Rd := Op1 AND Op2
    And = 0,
    // Rd := Op1 EOR Op2
    ExclusiveOr = 1,
    // Rd := Op1 - Op2
    Subtract = 2,
    // Rd := Op2 - Op1
    ReverseSubtract = 3,
    // Rd := Op1 + Op2
    Add = 4,
    // Rd := Op1 + Op2 + C
    AddWithCarry = 5,
    // Rd := Op1 - Op2 + C - 1
    SubtractWithCarry = 6,
    // Rd := Op2 - Op1 + C - 1
    ReverseSubtractWithCarry = 7,
    // Set conditions on Op1 AND Op2
    Test = 8,
    // Set conditions on Op1 EOR Op2
    TestEqual = 9,
    // Set conditions on Op1 - Op2
    Compare = 10,
    // Set conditions on Op1 + Op2
    CompareNegate = 11,
    // Rd := Op1 OR Op2
    Or = 12,
    // Rd := Op2
    Move = 13,
    // Rd := Op1 AND NOT Op2
    AndNot = 14,
    // Rd := NOT Op2
    MoveNegate = 15,
}

pub struct DataProcessingInstruction {
    update_conditions: bool,
    source_register_index: u32,
    operand: Operand,
    destination_register_index: Option<u32>,
    operation: DataProcessingOperation,
}

impl DataProcessingInstruction {
    pub fn new(
        update_conditions: bool,
        source_register_index: u32,
        operand: Operand,
        destination_register_index: Option<u32>,
        operation: DataProcessingOperation,
    ) -> Self {
        Self {
            update_conditions,
            source_register_index,
            operand,
            destination_register_index,
            operation,
        }
    }

    pub fn decode(registers: &mut RegisterBank, opcode: u32) -> Self {
        let operand = if opcode & (1 << 25) > 0 {
            Operand::Immediate(rotated_immediate(opcode))
        } else {
            match Shift::from_opcode(opcode) {
                Shift::Immediate(shift) => Operand::Immediate(shift.shift(registers)),
                Shift::Register(shift) => Operand::RegisterShifted(shift),
            }
        };

        let source_register_index = (opcode >> 16) & 0xF;
        let operation = DataProcessingOperation::try_from(((opcode >> 21) & 0xF) as u8).unwrap();
        let destination_register_index = if operation != DataProcessingOperation::Test
            && operation != DataProcessingOperation::TestEqual
            && operation != DataProcessingOperation::Compare
            && operation != DataProcessingOperation::CompareNegate
        {
            Some((opcode >> 12) & 0xF)
        } else {
            None
        };

        Self {
            update_conditions: opcode & (1 << 20) > 0,
            source_register_index,
            operand,
            operation,
            destination_register_index,
        }
    }
}

impl InstructionExecutor for DataProcessingInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        let source = registers.reg(self.source_register_index as usize);
        let operand = self.operand.value(registers);
        let (result, overflow) = match self.operation {
            DataProcessingOperation::And => (source & operand, false),
            DataProcessingOperation::Test => (source & operand, false),
            DataProcessingOperation::ExclusiveOr => (source ^ operand, false),
            DataProcessingOperation::TestEqual => (source ^ operand, false),
            DataProcessingOperation::Subtract => {
                let (result, overflow) = source.overflowing_sub(operand);
                (result, overflow)
            }
            DataProcessingOperation::ReverseSubtract => {
                let (result, overflow) = operand.overflowing_sub(source);
                (result, overflow)
            }
            DataProcessingOperation::Add => {
                let (result, overflow) = source.overflowing_add(operand);
                (result, overflow)
            }
            DataProcessingOperation::AddWithCarry => {
                let (result, overflow1) = source.overflowing_add(operand);
                let (result, overflow2) = result.overflowing_add(registers.cpsr.carry as u32);
                (result, overflow1 || overflow2)
            }
            DataProcessingOperation::SubtractWithCarry => {
                let (result, overflow1) = source.overflowing_sub(operand);
                let (result, overflow2) = result.overflowing_add(registers.cpsr.carry as u32 - 1);
                (result, overflow1 || overflow2)
            }
            DataProcessingOperation::ReverseSubtractWithCarry => {
                let (result, overflow1) = operand.overflowing_sub(source);
                let (result, overflow2) = result.overflowing_add(registers.cpsr.carry as u32 - 1);
                (result, overflow1 || overflow2)
            }
            DataProcessingOperation::Compare => {
                let (result, overflow) = source.overflowing_sub(operand);
                (result, overflow)
            }
            DataProcessingOperation::CompareNegate => {
                let (result, overflow) = source.overflowing_add(operand);
                (result, overflow)
            }
            DataProcessingOperation::Or => (source | operand, false),
            DataProcessingOperation::Move => (operand, false),
            DataProcessingOperation::AndNot => (source & !operand, false),
            DataProcessingOperation::MoveNegate => (!operand, false),
        };

        if let Some(destination_register_index) = self.destination_register_index {
            *registers.reg_mut(destination_register_index as usize) = result;
        }

        // Check if condition code should be updated.
        if self.update_conditions {
            registers.cpsr.overflow =
                ((source ^ operand) & 0x80000000 == 0) && ((source ^ result) & 0x80000000 != 0);
            registers.cpsr.carry = overflow;
            registers.cpsr.zero = result == 0;
            registers.cpsr.signed = result & (1 << 31) > 0;
        }

        // TODO: Calculate cycle timings.
        Ok(1)
    }

    fn mnemonic(&self) -> String {
        let mnemonic = match self.operation {
            DataProcessingOperation::And => "and",
            DataProcessingOperation::Test => "tst",
            DataProcessingOperation::ExclusiveOr => "eor",
            DataProcessingOperation::TestEqual => "teq",
            DataProcessingOperation::Subtract => "sub",
            DataProcessingOperation::ReverseSubtract => "rsb",
            DataProcessingOperation::Add => "add",
            DataProcessingOperation::AddWithCarry => "adc",
            DataProcessingOperation::SubtractWithCarry => "sbc",
            DataProcessingOperation::ReverseSubtractWithCarry => "rsc",
            DataProcessingOperation::Compare => "cmp",
            DataProcessingOperation::CompareNegate => "cmn",
            DataProcessingOperation::Or => "orr",
            DataProcessingOperation::Move => "mov",
            DataProcessingOperation::AndNot => "bic",
            DataProcessingOperation::MoveNegate => "mvn",
        };
        format!(
            "{mnemonic}{}",
            if self.update_conditions { "s" } else { "" }
        )
        .into()
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        if let Some(destination_register_index) = self.destination_register_index {
            format!(
                "r{destination_register_index}, r{}, {}",
                self.source_register_index, self.operand
            )
        } else {
            format!("r{}, {}", self.source_register_index, self.operand)
        }
    }
}
