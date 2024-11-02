use std::fmt::Display;

use crate::core::{Bus, CoreError};

use super::register::RegisterBank;
use super::shift::Shift;
use super::thumb::LongBranchWithLinkInstruction;

use super::arm::{
    BlockDataTransferInstruction, BranchAndExchangeInstruction, BranchInstruction,
    DataProcessingInstruction, PsrTransferMrsInstruction, PsrTransferMsrInstruction,
    SingleDataSwapInstruction, SingleDataTransferInstruction, SoftwareInterruptInstruction,
};

pub trait InstructionExecutor {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError>;
    fn mnemonic(&self) -> String;
    fn description(&self, registers: &RegisterBank, bus: &mut Bus) -> String;
}

pub enum Instruction {
    Branch(BranchInstruction),
    BranchAndExchange(BranchAndExchangeInstruction),
    LongBranchWithLink(LongBranchWithLinkInstruction),
    DataProcessing(DataProcessingInstruction),
    SingleDataTransfer(SingleDataTransferInstruction),
    SoftwareInterrupt(SoftwareInterruptInstruction),
    BlockDataTransfer(BlockDataTransferInstruction),
    PsrTransferMrs(PsrTransferMrsInstruction),
    PsrTransferMsr(PsrTransferMsrInstruction),
    SingleDataSwap(SingleDataSwapInstruction),
}

pub struct Operation {
    pub location: u32,
    pub opcode: u32,
    pub condition: u32,
    pub instruction: Instruction,
}

pub enum Operand {
    Immediate(u32),
    Register(u32),
    RegisterShifted(Shift),
}

impl Operand {
    pub fn value(&self, registers: &RegisterBank) -> u32 {
        match self {
            Operand::Immediate(value) => *value,
            Operand::Register(index) => registers.reg(*index as usize),
            Operand::RegisterShifted(shift) => shift.shift(registers),
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(value) => write!(f, "#0x{:X}", value),
            Operand::Register(value) => write!(f, "r{}", value),
            Operand::RegisterShifted(shift) => write!(f, "{}", shift),
        }
    }
}
