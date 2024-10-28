mod arithmetic;
mod branch;
mod disasm;
mod instruction;
mod interrupt;
mod multiply;
mod register;
mod shift;
mod status;
mod transfer;

use interrupt::{SoftwareInterruptInstruction, SOFTWARE_INTERRUPT_FORMAT, SOFTWARE_INTERRUPT_MASK};
use multiply::{MULTIPLY_FORMAT, MULTIPLY_LONG_FORMAT, MULTIPLY_MASK};
use transfer::{
    BlockDataTransferInstruction, PsrTransferMrsInstruction, PsrTransferMsrInstruction,
    SingleDataSwapInstruction, BLOCK_TRANSFER_FORMAT, BLOCK_TRANSFER_MASK, PSR_TRANSFER_MRS_FORMAT,
    PSR_TRANSFER_MRS_MASK, PSR_TRANSFER_MSR_FORMAT, PSR_TRANSFER_MSR_MASK, SINGLE_DATA_SWAP_FORMAT,
    SINGLE_DATA_SWAP_MASK,
};

use self::{
    arithmetic::{DataProcessingInstruction, DATA_PROCESSING_FORMAT, DATA_PROCESSING_MASK},
    branch::{
        BranchAndExchangeInstruction, BranchInstruction, BRANCH_AND_EXCHANGE_FORMAT,
        BRANCH_AND_EXCHANGE_MASK, BRANCH_FORMAT, BRANCH_MASK,
    },
    instruction::{Instruction, InstructionExecutor, Operation},
    register::RegisterBank,
    status::InstructionMode,
    transfer::{SingleDataTransferInstruction, SINGLE_TRANSFER_FORMAT, SINGLE_TRANSFER_MASK},
};

use super::{Bus, CoreError};

#[derive(Default)]
pub struct Interpreter {
    registers: RegisterBank,
    fetched_instruction: Option<(u32, u32)>,
    decoded_instruction: Option<Operation>,
}

impl Interpreter {
    pub fn tick(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        let cycles = self.execute(bus)?;
        self.decode()?;
        self.fetch(bus)?;
        Ok(cycles)
    }

    fn fetch(&mut self, bus: &mut Bus) -> Result<(), CoreError> {
        let fetch_location = match self.registers.cpsr.instruction_mode {
            InstructionMode::Arm => self.registers.pc(),
            InstructionMode::Thumb => self.registers.pc(),
        };
        self.fetched_instruction = Some((bus.read_dword(fetch_location)?, fetch_location));
        match self.registers.cpsr.instruction_mode {
            InstructionMode::Arm => *self.registers.pc_mut() += 4,
            InstructionMode::Thumb => *self.registers.pc_mut() += 2,
        }
        Ok(())
    }

    fn decode(&mut self) -> Result<(), CoreError> {
        match self.registers.cpsr.instruction_mode {
            InstructionMode::Arm => self.decode_arm(),
            InstructionMode::Thumb => self.decode_thumb(),
        }
    }

    fn decode_arm(&mut self) -> Result<(), CoreError> {
        if let Some((fetched_instruction, pc)) = self.fetched_instruction {
            self.decoded_instruction = Some(Operation {
                location: pc,
                condition: fetched_instruction >> 28,
                opcode: fetched_instruction,
                instruction: if (fetched_instruction & BRANCH_AND_EXCHANGE_MASK)
                    == BRANCH_AND_EXCHANGE_FORMAT
                {
                    Instruction::BranchAndExchange(BranchAndExchangeInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & BLOCK_TRANSFER_MASK) == BLOCK_TRANSFER_FORMAT {
                    Instruction::BlockDataTransfer(BlockDataTransferInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & BRANCH_MASK) == BRANCH_FORMAT {
                    Instruction::Branch(BranchInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & SOFTWARE_INTERRUPT_MASK)
                    == SOFTWARE_INTERRUPT_FORMAT
                {
                    Instruction::SoftwareInterrupt(SoftwareInterruptInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & SINGLE_TRANSFER_MASK) == SINGLE_TRANSFER_FORMAT {
                    Instruction::SingleDataTransfer(SingleDataTransferInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & SINGLE_DATA_SWAP_MASK) == SINGLE_DATA_SWAP_FORMAT {
                    Instruction::SingleDataSwap(SingleDataSwapInstruction::decode(
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & MULTIPLY_MASK) == MULTIPLY_FORMAT {
                    unimplemented!()
                } else if (fetched_instruction & MULTIPLY_MASK) == MULTIPLY_LONG_FORMAT {
                    unimplemented!()
                } else if (fetched_instruction & PSR_TRANSFER_MRS_MASK) == PSR_TRANSFER_MRS_FORMAT {
                    Instruction::PsrTransferMrs(PsrTransferMrsInstruction::decode(
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & PSR_TRANSFER_MSR_MASK) == PSR_TRANSFER_MSR_FORMAT {
                    Instruction::PsrTransferMsr(PsrTransferMsrInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & DATA_PROCESSING_MASK) == DATA_PROCESSING_FORMAT {
                    Instruction::DataProcessing(DataProcessingInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else {
                    return Err(CoreError::OpcodeNotImplemented(fetched_instruction));
                },
            });
        }
        Ok(())
    }

    fn decode_thumb(&mut self) -> Result<(), CoreError> {
        if let Some((fetched_instruction, pc)) = self.fetched_instruction {
            let fetched_instruction = fetched_instruction & 0xFFFF;
            return Err(CoreError::OpcodeNotImplemented(fetched_instruction));
        }

        Ok(())
    }

    fn execute(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        match self.registers.cpsr.instruction_mode {
            InstructionMode::Arm => self.tick_arm(bus),
            InstructionMode::Thumb => self.tick_thumb(bus),
        }
    }

    fn tick_arm(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        if let Some(decoded_instruction) = &self.decoded_instruction {
            let ins: &dyn InstructionExecutor = match &decoded_instruction.instruction {
                Instruction::Branch(b) => b,
                Instruction::BranchAndExchange(b) => b,
                Instruction::DataProcessing(d) => d,
                Instruction::SingleDataTransfer(d) => d,
                Instruction::SoftwareInterrupt(i) => i,
                Instruction::BlockDataTransfer(d) => d,
                Instruction::PsrTransferMrs(d) => d,
                Instruction::PsrTransferMsr(d) => d,
                Instruction::SingleDataSwap(d) => d,
            };

            self.log_instruction(
                decoded_instruction.location,
                decoded_instruction.opcode,
                &ins.mnemonic(),
                &ins.description(),
            );

            if self.check_condition(decoded_instruction.condition) {
                let original_pc = self.registers.pc();
                let cycles = ins.execute(&mut self.registers, bus);
                if self.registers.pc() != original_pc {
                    self.decoded_instruction = None;
                    self.fetched_instruction = None;
                }
                return cycles;
            }
        }

        Ok(1)
    }

    fn tick_thumb(&self, _bus: &mut Bus) -> Result<usize, CoreError> {
        Ok(1)
    }

    pub fn log_instruction(&self, address: u32, opcode: u32, mneumonic: &str, description: &str) {
        let condition = Self::get_condition_label(opcode >> 28);
        println!(
            "${address:08X}: {opcode:08X} {mneumonic}{}{condition} {description}",
            if condition.len() > 0 { "." } else { "" },
        );
    }

    fn get_condition_label(condition_code: u32) -> &'static str {
        match condition_code {
            0x0 => "eq",
            0x1 => "ne",
            0x2 => "cs",
            0x3 => "cc",
            0x4 => "mi",
            0x5 => "pl",
            0x6 => "vs",
            0x7 => "vc",
            0x8 => "hi",
            0x9 => "ls",
            0xA => "ge",
            0xB => "lt",
            0xC => "gt",
            0xD => "le",
            0xE => "",
            0xF => "nv",
            _ => unreachable!(),
        }
    }

    fn check_condition(&self, condition: u32) -> bool {
        match condition {
            0x0 => self.registers.cpsr.zero,
            0x1 => !self.registers.cpsr.zero,
            0x2 => self.registers.cpsr.carry,
            0x3 => !self.registers.cpsr.carry,
            0x4 => self.registers.cpsr.signed,
            0x5 => !self.registers.cpsr.signed,
            0x6 => self.registers.cpsr.overflow,
            0x7 => !self.registers.cpsr.overflow,
            0x8 => self.registers.cpsr.carry && !self.registers.cpsr.zero,
            0x9 => !self.registers.cpsr.carry || self.registers.cpsr.zero,
            0xA => self.registers.cpsr.signed == self.registers.cpsr.overflow,
            0xB => self.registers.cpsr.signed != self.registers.cpsr.overflow,
            0xC => {
                !self.registers.cpsr.zero
                    && self.registers.cpsr.signed == self.registers.cpsr.overflow
            }
            0xD => {
                self.registers.cpsr.zero
                    || self.registers.cpsr.signed != self.registers.cpsr.overflow
            }
            0xE => true,
            0xF => false,
            _ => true,
        }
    }
}
