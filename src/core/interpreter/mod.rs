mod arithmetic;
mod branch;
mod disasm;
mod instruction;
//mod interrupt;
//mod multiply;
mod register;
mod shift;
mod status;
mod transfer;

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
        let fetch_location = *self.registers.pc_mut() - 4;
        self.fetched_instruction = Some((bus.read_dword(fetch_location)?, fetch_location));
        *self.registers.pc_mut() += 4;
        Ok(())
    }

    fn decode(&mut self) -> Result<(), CoreError> {
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
                } else if (fetched_instruction & BRANCH_MASK) == BRANCH_FORMAT {
                    Instruction::Branch(BranchInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & SINGLE_TRANSFER_MASK) == SINGLE_TRANSFER_FORMAT {
                    Instruction::SingleDataTransfer(SingleDataTransferInstruction::decode(
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

    fn execute(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        match self.registers.cpsr.instruction_mode {
            InstructionMode::Arm => self.tick_arm(bus),
            InstructionMode::Thumb => unimplemented!(),
        }
    }

    fn tick_arm(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        if let Some(decoded_instruction) = &self.decoded_instruction {
            let ins: &dyn InstructionExecutor = match &decoded_instruction.instruction {
                Instruction::Branch(b) => b,
                Instruction::BranchAndExchange(b) => b,
                Instruction::DataProcessing(d) => d,
                Instruction::SingleDataTransfer(d) => d,
            };

            self.log_instruction(
                decoded_instruction.location,
                decoded_instruction.opcode,
                &ins.mneumonic(),
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
        /*if (opcode & BRANCH_AND_EXCHANGE_MASK) == BRANCH_AND_EXCHANGE_FORMAT {
            Ok(self.branch_and_exchange(opcode))
        } else if (opcode & BLOCK_TRANSFER_MASK) == BLOCK_TRANSFER_FORMAT {
            Ok(self.block_data_transfer(opcode, bus)?)
        } else if (opcode & BRANCH_MASK) == BRANCH_FORMAT {
            Ok(self.branch(opcode))
        } else if (opcode & SOFTWARE_INTERRUPT_MASK) == SOFTWARE_INTERRUPT_FORMAT {
            Ok(self.software_interrupt(opcode))
        } else if (opcode & SINGLE_TRANSFER_MASK) == SINGLE_TRANSFER_FORMAT {
            Ok(self.single_data_transfer(opcode, bus)?)
        } else if (opcode & SINGLE_DATA_SWAP_MASK) == SINGLE_DATA_SWAP_FORMAT {
            Ok(self.single_data_swap(opcode, bus)?)
        } else if (opcode & MULTIPLY_MASK) == MULTIPLY_FORMAT {
            Ok(self.multiply(opcode))
        } else if (opcode & MULTIPLY_MASK) == MULTIPLY_LONG_FORMAT {
            Ok(self.multiply_long(opcode))
        } else if (opcode & PSR_TRANSFER_MRS_MASK) == PSR_TRANSFER_MRS_FORMAT {
            Ok(self.psr_transfer_mrs(opcode))
        } else if (opcode & PSR_TRANSFER_MSR_MASK) == PSR_TRANSFER_MSR_FORMAT {
            Ok(self.psr_transfer_msr(opcode))
        } else if (opcode & DATA_PROCESSING_MASK) == DATA_PROCESSING_FORMAT {
            Ok(self.process_data(opcode))
        } else {
            Err(CoreError::OpcodeNotImplemented(opcode))
        }*/
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
