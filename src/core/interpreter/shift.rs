use std::fmt::Display;

use super::register::RegisterBank;

const SHIFT_TYPE_LSL: u32 = 0;
const SHIFT_TYPE_LSR: u32 = 1;
const SHIFT_TYPE_ASR: u32 = 2;
const SHIFT_TYPE_ROR: u32 = 3;

pub struct RegisterShift {
    base_register: u32,
    shift_register: u32,
    shift_type: ShiftType,
}

pub struct ImmediateShift {
    base_register: u32,
    shift_amount: u32,
    shift_type: ShiftType,
}

pub enum Shift {
    Register(RegisterShift),
    Immediate(ImmediateShift),
}

enum ShiftType {
    LogicalLeft,
    LogicalRight,
    ArithmeticRight,
    RotateRight,
}

impl ShiftType {
    pub fn from_u32(value: u32) -> Self {
        match value & 0b11 {
            SHIFT_TYPE_LSL => ShiftType::LogicalLeft,
            SHIFT_TYPE_LSR => ShiftType::LogicalRight,
            SHIFT_TYPE_ASR => ShiftType::ArithmeticRight,
            SHIFT_TYPE_ROR => ShiftType::RotateRight,
            _ => unreachable!(),
        }
    }

    pub fn shift(&self, operand: u32, shift_amount: u32) -> u32 {
        match self {
            ShiftType::LogicalLeft => operand << shift_amount,
            ShiftType::LogicalRight => operand >> shift_amount,
            ShiftType::ArithmeticRight => ((operand as i32) >> shift_amount) as u32,
            ShiftType::RotateRight => operand.rotate_right(shift_amount),
        }
    }
}

impl Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftType::LogicalLeft => write!(f, "LSL"),
            ShiftType::LogicalRight => write!(f, "LSR"),
            ShiftType::ArithmeticRight => write!(f, "ASR"),
            ShiftType::RotateRight => write!(f, "ROR"),
        }
    }
}

impl Shift {
    pub fn from_opcode(opcode: u32) -> Self {
        if opcode & (1 << 4) > 0 {
            Shift::Register(RegisterShift {
                base_register: opcode & 0xF,
                shift_register: (opcode >> 8) & 0xF,
                shift_type: ShiftType::from_u32((opcode >> 5) & 0b11),
            })
        } else {
            Shift::Immediate(ImmediateShift {
                base_register: opcode & 0xF,
                shift_amount: (opcode >> 7) & 0x1F,
                shift_type: ShiftType::from_u32((opcode >> 5) & 0b11),
            })
        }
    }
}

impl ImmediateShift {
    pub fn shift(&self, registers: &RegisterBank) -> u32 {
        self.shift_type.shift(
            registers.reg(self.base_register as usize),
            self.shift_amount,
        )
    }
}

impl RegisterShift {
    pub fn shift(&self, registers: &RegisterBank) -> u32 {
        self.shift_type.shift(
            registers.reg(self.base_register as usize),
            registers.reg(self.shift_register as usize),
        )
    }
}

impl Display for RegisterShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "r{}, {}, r{}",
            self.base_register, self.shift_type, self.shift_register
        )
    }
}

pub fn rotated_immediate(opcode: u32) -> u32 {
    let shift_amount = 2 * ((opcode >> 8) & 0xF);
    let immediate = opcode & 0xFF;
    (immediate as u32).rotate_right(shift_amount)
}
