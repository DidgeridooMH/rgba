use std::fmt::Display;

use crate::core::{Bus, CoreError};

use super::{register::RegisterBank, shift::RegisterShift};

use super::arm::{
    BlockDataTransferInstruction, BranchAndExchangeInstruction, BranchInstruction,
    DataProcessingInstruction, PsrTransferMrsInstruction, PsrTransferMsrInstruction,
    SingleDataSwapInstruction, SingleDataTransferInstruction, SoftwareInterruptInstruction,
};

pub trait InstructionExecutor {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError>;
    fn mnemonic(&self) -> String;
    fn description(&self) -> String;
}

pub enum Instruction {
    // Arm Instructions
    Branch(BranchInstruction),
    BranchAndExchange(BranchAndExchangeInstruction),
    DataProcessing(DataProcessingInstruction),
    SingleDataTransfer(SingleDataTransferInstruction),
    SoftwareInterrupt(SoftwareInterruptInstruction),
    BlockDataTransfer(BlockDataTransferInstruction),
    PsrTransferMrs(PsrTransferMrsInstruction),
    PsrTransferMsr(PsrTransferMsrInstruction),
    SingleDataSwap(SingleDataSwapInstruction),
    // Thumb Instructions
    ThumbSoftwareInterrupt,
    ThumbUnconditionalBranch,
    ThumbConditionalBranch,
    ThumbMultipleLoadStore,
    ThumbLongBranchWithLink,
    AddOffsetToStackPointer,
    PushPopRegisters,
    LoadStoreHalfword,
    SpRelativeLoadStore,
    LoadAddress,
    LoadStoreWithImmediateOffset,
    LoadStorewithRegisterOffset,
    LoadStoreSignExtByteHalfword,
    PcRelativeLoad,
    HiRegisterOperationsBranchExchange,
    AluOperation,
    MoveCompareAddSubtractImmediate,
    AddSubtract,
    MoveShiftedRegister,
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
