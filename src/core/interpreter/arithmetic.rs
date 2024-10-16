use super::{Interpreter, OperandType};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

pub const DATA_PROCESSING_MASK: u32 = 0x0C000000;
pub const DATA_PROCESSING_FORMAT: u32 = 0x00000000;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum DataProcessingOperation {
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

impl Interpreter {
    pub fn process_data(&mut self, opcode: u32) -> usize {
        let operand_type = if opcode & (1 << 25) > 0 {
            OperandType::Immediate
        } else {
            OperandType::Register
        };

        let operand = match operand_type {
            OperandType::Immediate => {
                let shift_amount = 2 * ((opcode >> 8) & 0xF);
                let immediate = opcode & 0xFF;
                (immediate as u32).rotate_right(shift_amount)
            }
            OperandType::Register => self.shift_operand(opcode),
        };

        let source_register_index: usize = ((opcode >> 16) & 0xF) as usize;

        let operation = DataProcessingOperation::try_from(((opcode >> 21) & 0xF) as u8).unwrap();

        let destination_register_index = (opcode >> 12) & 0xF;
        let source = *self.reg_mut(source_register_index);
        let (result, overflow, mneumonic, description) = match operation {
            DataProcessingOperation::And => (
                source & operand,
                false,
                "AND",
                format!("r{source_register_index}({source:X}) & {operand:X}"),
            ),
            DataProcessingOperation::Test => (
                source & operand,
                false,
                "TEST",
                format!("r{source_register_index}({source:X}) & {operand:X}"),
            ),
            DataProcessingOperation::ExclusiveOr => (
                source ^ operand,
                false,
                "EOR",
                format!("r{source_register_index}({source:X}) ^ {operand:X}"),
            ),
            DataProcessingOperation::TestEqual => (
                source ^ operand,
                false,
                "TEQ",
                format!("r{source_register_index}({source:X}) ^ {operand:X}"),
            ),
            DataProcessingOperation::Subtract => {
                let (result, overflow) = source.overflowing_sub(operand);
                (
                    result,
                    overflow,
                    "SUB",
                    format!("r{source_register_index}({source:X}) - {operand:X}"),
                )
            }
            DataProcessingOperation::ReverseSubtract => {
                let (result, overflow) = operand.overflowing_sub(source);
                (
                    result,
                    overflow,
                    "RSB",
                    format!("{operand:X} - r{source_register_index}({source:X})"),
                )
            }
            DataProcessingOperation::Add => {
                let (result, overflow) = source.overflowing_add(operand);
                (
                    result,
                    overflow,
                    "ADD",
                    format!("r{source_register_index}({source:X}) + {operand:X}"),
                )
            }
            DataProcessingOperation::AddWithCarry => {
                // TODO: Check if this needs to account for a double carry.
                let (result, _) = source.overflowing_add(operand);
                let (result, overflow) = result.overflowing_add(self.cpsr.carry as u32);
                (
                    result,
                    overflow,
                    "ADC",
                    format!("r{source_register_index}({source:X}) + {operand:X} + C"),
                )
            }
            DataProcessingOperation::SubtractWithCarry => {
                let (result, _) = source.overflowing_sub(operand);
                let (result, overflow) = result.overflowing_add(self.cpsr.carry as u32 - 1);
                (
                    result,
                    overflow,
                    "SBC",
                    format!("r{source_register_index}({source:X}) - {operand:X} + C - 1"),
                )
            }
            DataProcessingOperation::ReverseSubtractWithCarry => {
                let (result, _) = operand.overflowing_sub(source);
                let (result, overflow) = result.overflowing_add(self.cpsr.carry as u32 - 1);
                (
                    result,
                    overflow,
                    "RSC",
                    format!("{operand:X} - r{source_register_index}({source:X}) + C - 1"),
                )
            }
            DataProcessingOperation::Compare => {
                let (result, overflow) = source.overflowing_sub(operand);
                (
                    result,
                    overflow,
                    "CMP",
                    format!("r{source_register_index}({source:X}) - {operand:X}"),
                )
            }
            DataProcessingOperation::CompareNegate => {
                let (result, overflow) = source.overflowing_add(operand);
                (
                    result,
                    overflow,
                    "CMN",
                    format!("r{source_register_index}({source:X}) + {operand:X}"),
                )
            }
            DataProcessingOperation::Or => (
                source | operand,
                false,
                "ORR",
                format!("r{source_register_index}({source:X}) | {operand:X}"),
            ),
            DataProcessingOperation::Move => (operand, false, "MOV", format!("{operand:X}")),
            DataProcessingOperation::AndNot => (
                source & !operand,
                false,
                "BIC",
                format!("r{source_register_index}({source:X}) & !{operand:X}"),
            ),
            DataProcessingOperation::MoveNegate => {
                (!operand, false, "MVN", format!("!{operand:X}"))
            }
        };

        self.log_instruction(
            opcode,
            mneumonic,
            &format!("r{destination_register_index}({result:X}) := {description}",),
        );

        if operation != DataProcessingOperation::Test
            && operation != DataProcessingOperation::TestEqual
            && operation != DataProcessingOperation::Compare
            && operation != DataProcessingOperation::CompareNegate
        {
            *self.reg_mut(destination_register_index as usize) = result;
        }

        // Check if condition code should be updated.
        if opcode & (1 << 20) > 0 {
            self.cpsr.overflow = overflow;
            self.cpsr.carry = source >= operand;
            self.cpsr.zero = result == 0;
            self.cpsr.signed = result & (1 << 31) > 0;
        }

        // TODO: Calculate cycle timings.
        1
    }
}
