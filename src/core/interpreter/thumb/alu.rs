use num_enum::TryFromPrimitive;

use crate::core::interpreter::{
    arm::{DataProcessingInstruction, DataProcessingOperation},
    instruction::{Instruction, Operand},
    shift::{ImmediateShift, RegisterShift, Shift, ShiftType},
};

pub const MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_FORMAT: u32 = 0b0010_0000_0000_0000;
pub const MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_MASK: u32 = 0b1110_0000_0000_0000;

pub const ADD_SUBTRACT_FORMAT: u32 = 0b0001_1000_0000_0000;
pub const ADD_SUBTRACT_MASK: u32 = 0b1111_1000_0000_0000;

pub const ALU_OPERATION_FORMAT: u32 = 0b0100_0000_0000_0000;
pub const ALU_OPERATION_MASK: u32 = 0b1111_1100_0000_0000;

pub const MOVE_SHIFTED_REGISTER_FORMAT: u32 = 0b0000_0000_0000_0000;
pub const MOVE_SHIFTED_REGISTER_MASK: u32 = 0b1110_0000_0000_0000;

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
    let imm8 = Operand::Immediate((opcode & 0xFF, false));

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
        Operand::Immediate((rn as u32, false))
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

#[derive(TryFromPrimitive)]
#[repr(u32)]
enum AluOperation {
    And = 0,
    Eor = 1,
    Lsl = 2,
    Lsr = 3,
    Asr = 4,
    Adc = 5,
    Sbc = 6,
    Ror = 7,
    Tst = 8,
    Neg = 9,
    Cmp = 10,
    Cmn = 11,
    Orr = 12,
    Mul = 13,
    Bic = 14,
    Mvn = 15,
}

pub fn decode_alu_operations(opcode: u32) -> Instruction {
    let operation = AluOperation::try_from((opcode >> 6) & 0b1111).unwrap();
    let rs = (opcode >> 3) & 0b111;
    let rd = opcode & 0b111;

    let (op, operand) = match operation {
        AluOperation::And => (DataProcessingOperation::And, Operand::Register(rs)),
        AluOperation::Eor => (DataProcessingOperation::ExclusiveOr, Operand::Register(rs)),
        AluOperation::Lsl => (
            DataProcessingOperation::Move,
            Operand::RegisterShifted(Shift::Register(RegisterShift::new(
                rd,
                rs,
                ShiftType::LogicalLeft,
            ))),
        ),
        AluOperation::Lsr => (
            DataProcessingOperation::Move,
            Operand::RegisterShifted(Shift::Register(RegisterShift::new(
                rd,
                rs,
                ShiftType::LogicalRight,
            ))),
        ),
        AluOperation::Asr => (
            DataProcessingOperation::Move,
            Operand::RegisterShifted(Shift::Register(RegisterShift::new(
                rd,
                rs,
                ShiftType::ArithmeticRight,
            ))),
        ),
        AluOperation::Adc => (DataProcessingOperation::AddWithCarry, Operand::Register(rs)),
        AluOperation::Sbc => (
            DataProcessingOperation::SubtractWithCarry,
            Operand::Register(rs),
        ),
        AluOperation::Ror => (
            DataProcessingOperation::Move,
            Operand::RegisterShifted(Shift::Register(RegisterShift::new(
                rd,
                rs,
                ShiftType::RotateRight,
            ))),
        ),
        AluOperation::Tst => (DataProcessingOperation::Test, Operand::Register(rs)),
        AluOperation::Neg => (
            DataProcessingOperation::Subtract,
            Operand::Immediate((0, false)),
        ),
        AluOperation::Cmp => (DataProcessingOperation::Compare, Operand::Register(rs)),
        AluOperation::Cmn => (
            DataProcessingOperation::CompareNegate,
            Operand::Register(rs),
        ),
        AluOperation::Orr => (DataProcessingOperation::Or, Operand::Register(rs)),
        AluOperation::Mul => unimplemented!(),
        AluOperation::Bic => (DataProcessingOperation::AndNot, Operand::Register(rs)),
        AluOperation::Mvn => (DataProcessingOperation::MoveNegate, Operand::Register(rs)),
    };

    let source = match operation {
        AluOperation::Neg => rs,
        _ => rd,
    };

    let destination = match operation {
        AluOperation::Tst | AluOperation::Cmp | AluOperation::Cmn => None,
        _ => Some(rd as u32),
    };

    Instruction::DataProcessing(DataProcessingInstruction::new(
        true,
        source,
        operand,
        destination,
        op,
    ))
}

pub fn decode_move_shifted_register(opcode: u32) -> Instruction {
    let op = ShiftType::from_u32((opcode >> 11) & 0b11);
    let offset = (opcode >> 6) & 0b11111;
    let rs = (opcode >> 3) & 0b111;
    let rd = opcode & 0b111;

    let shift_type = match op {
        ShiftType::LogicalLeft => ShiftType::LogicalLeft,
        ShiftType::LogicalRight => ShiftType::LogicalRight,
        ShiftType::ArithmeticRight => ShiftType::ArithmeticRight,
        ShiftType::RotateRight => ShiftType::RotateRight,
    };

    Instruction::DataProcessing(DataProcessingInstruction::new(
        true,
        rs,
        Operand::RegisterShifted(Shift::Immediate(ImmediateShift::new(
            rs, offset, shift_type,
        ))),
        Some(rd as u32),
        DataProcessingOperation::Move,
    ))
}
