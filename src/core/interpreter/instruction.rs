use std::fmt::Display;

use crate::core::{Bus, CoreError};

use super::{
    arithmetic::DataProcessingInstruction,
    branch::{BranchAndExchangeInstruction, BranchInstruction},
    interrupt::SoftwareInterruptInstruction,
    register::RegisterBank,
    shift::RegisterShift,
    transfer::{
        BlockDataTransferInstruction, PsrTransferMrsInstruction, PsrTransferMsrInstruction,
        SingleDataSwapInstruction, SingleDataTransferInstruction,
    },
};

pub trait InstructionExecutor {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError>;
    fn mnemonic(&self) -> String;
    fn description(&self) -> String;
}

pub enum Instruction {
    Branch(BranchInstruction),
    BranchAndExchange(BranchAndExchangeInstruction),
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
    RegisterShifted(RegisterShift),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(value) => write!(f, "#0x{:X}", value),
            Operand::RegisterShifted(shift) => write!(f, "{}", shift),
        }
    }
}
