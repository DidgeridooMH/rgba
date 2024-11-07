mod arm;
mod disasm;
mod instruction;
mod register;
mod shift;
mod status;
mod thumb;

use instruction::{Instruction, InstructionExecutor, Operation};
use register::RegisterBank;
use status::InstructionMode;
use thumb::{
    decode_add_offset_stack_pointer, decode_add_subtract, decode_alu_operations,
    decode_conditional_branch, decode_hi_reg_branch_exchange, decode_load_store_halfword,
    decode_move_shifted_register, decode_push_pop_registers, decode_sp_relative_load_store,
    LongBranchWithLinkInstruction,
};

use super::{Bus, CoreError};

#[derive(Default)]
pub struct Interpreter {
    registers: RegisterBank,
    fetched_instruction: Option<(u32, u32)>,
    decoded_instruction: Option<Operation>,
    pub logging_enabled: bool,
}

impl Interpreter {
    pub fn tick(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        let cycles = self.execute(bus)?;
        self.decode()?;
        self.fetch(bus)?;
        Ok(cycles)
    }

    fn fetch(&mut self, bus: &mut Bus) -> Result<(), CoreError> {
        let fetch_location = self.registers.pc();
        self.fetched_instruction = Some((bus.read_dword(fetch_location)?, fetch_location));
        self.registers.increment_pc();
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
                instruction: if (fetched_instruction & arm::BRANCH_AND_EXCHANGE_MASK)
                    == arm::BRANCH_AND_EXCHANGE_FORMAT
                {
                    Instruction::BranchAndExchange(arm::BranchAndExchangeInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::BLOCK_TRANSFER_MASK)
                    == arm::BLOCK_TRANSFER_FORMAT
                {
                    Instruction::BlockDataTransfer(arm::BlockDataTransferInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::BRANCH_MASK) == arm::BRANCH_FORMAT {
                    Instruction::Branch(arm::BranchInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::SOFTWARE_INTERRUPT_MASK)
                    == arm::SOFTWARE_INTERRUPT_FORMAT
                {
                    Instruction::SoftwareInterrupt(arm::SoftwareInterruptInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::SINGLE_TRANSFER_MASK)
                    == arm::SINGLE_TRANSFER_FORMAT
                {
                    Instruction::SingleDataTransfer(arm::SingleDataTransferInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::SINGLE_DATA_SWAP_MASK)
                    == arm::SINGLE_DATA_SWAP_FORMAT
                {
                    Instruction::SingleDataSwap(arm::SingleDataSwapInstruction::decode(
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::MULTIPLY_MASK) == arm::MULTIPLY_FORMAT {
                    unimplemented!()
                } else if (fetched_instruction & arm::MULTIPLY_MASK) == arm::MULTIPLY_LONG_FORMAT {
                    unimplemented!()
                } else if (fetched_instruction & arm::HALFWORD_DATA_TRANSFER_REG_MASK)
                    == arm::HALFWORD_DATA_TRANSFER_REG_FORMAT
                {
                    Instruction::HalfwordDataTransfer(
                        arm::HalfwordDataTransferRegInstruction::decode(fetched_instruction),
                    )
                } else if (fetched_instruction & arm::PSR_TRANSFER_MRS_MASK)
                    == arm::PSR_TRANSFER_MRS_FORMAT
                {
                    Instruction::PsrTransferMrs(arm::PsrTransferMrsInstruction::decode(
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::PSR_TRANSFER_MSR_MASK)
                    == arm::PSR_TRANSFER_MSR_FORMAT
                {
                    Instruction::PsrTransferMsr(arm::PsrTransferMsrInstruction::decode(
                        &mut self.registers,
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & arm::DATA_PROCESSING_MASK)
                    == arm::DATA_PROCESSING_FORMAT
                {
                    Instruction::DataProcessing(arm::DataProcessingInstruction::decode(
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
            self.decoded_instruction = Some(Operation {
                location: pc,
                condition: if (fetched_instruction & thumb::CONDITIONAL_BRANCH_MASK)
                    == thumb::CONDITIONAL_BRANCH_FORMAT
                {
                    (fetched_instruction >> 8) & 0b1111
                } else {
                    0xE
                },
                opcode: fetched_instruction,
                instruction: if (fetched_instruction & thumb::SOFTWARE_INTERRUPT_MASK)
                    == thumb::SOFTWARE_INTERRUPT_FORMAT
                {
                    unimplemented!()
                } else if (fetched_instruction & thumb::UNCONDITIONAL_BRANCH_MASK)
                    == thumb::UNCONDITIONAL_BRANCH_FORMAT
                {
                    unimplemented!()
                } else if (fetched_instruction & thumb::CONDITIONAL_BRANCH_MASK)
                    == thumb::CONDITIONAL_BRANCH_FORMAT
                {
                    decode_conditional_branch(fetched_instruction)
                } else if (fetched_instruction & thumb::MULTIPLE_LOAD_STORE_MASK)
                    == thumb::MULTIPLE_LOAD_STORE_FORMAT
                {
                    unimplemented!()
                } else if (fetched_instruction & thumb::LONG_BRANCH_WITH_LINK_MASK)
                    == thumb::LONG_BRANCH_WITH_LINK_FORMAT
                {
                    Instruction::LongBranchWithLink(LongBranchWithLinkInstruction::decode(
                        fetched_instruction,
                    ))
                } else if (fetched_instruction & thumb::ADD_OFFSET_TO_STACK_POINTER_MASK)
                    == thumb::ADD_OFFSET_TO_STACK_POINTER_FORMAT
                {
                    decode_add_offset_stack_pointer(fetched_instruction)
                } else if (fetched_instruction & thumb::PUSH_POP_REGISTERS_MASK)
                    == thumb::PUSH_POP_REGISTERS_FORMAT
                {
                    decode_push_pop_registers(fetched_instruction)
                } else if (fetched_instruction & thumb::LOAD_STORE_HALFWORD_MASK)
                    == thumb::LOAD_STORE_HALFWORD_FORMAT
                {
                    decode_load_store_halfword(fetched_instruction)
                } else if (fetched_instruction & thumb::SP_RELATIVE_LOAD_STORE_MASK)
                    == thumb::SP_RELATIVE_LOAD_STORE_FORMAT
                {
                    decode_sp_relative_load_store(fetched_instruction)
                } else if (fetched_instruction & thumb::LOAD_ADDRESS_MASK)
                    == thumb::LOAD_ADDRESS_FORMAT
                {
                    unimplemented!()
                } else if (fetched_instruction & thumb::LOAD_STORE_WITH_IMMEDIATE_OFFSET_MASK)
                    == thumb::LOAD_STORE_WITH_IMMEDIATE_OFFSET_FORMAT
                {
                    unimplemented!()
                } else if (fetched_instruction & thumb::LOAD_STORE_WITH_REGISTER_OFFSET_MASK)
                    == thumb::LOAD_STORE_WITH_REGISTER_OFFSET_FORMAT
                {
                    thumb::decode_load_store_register_offset(fetched_instruction)
                } else if (fetched_instruction & thumb::LOAD_STORE_SIGN_EXT_BYTE_HALFWORD_MASK)
                    == thumb::LOAD_STORE_SIGN_EXT_BYTE_HALFWORD_FORMAT
                {
                    thumb::decode_load_store_sign_extended(fetched_instruction)
                } else if (fetched_instruction & thumb::PC_RELATIVE_LOAD_MASK)
                    == thumb::PC_RELATIVE_LOAD_FORMAT
                {
                    thumb::decode_pc_relative_load(fetched_instruction)
                } else if (fetched_instruction & thumb::HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_MASK)
                    == thumb::HI_REGISTER_OPERATIONS_BRANCH_EXCHANGE_FORMAT
                {
                    decode_hi_reg_branch_exchange(fetched_instruction)
                } else if (fetched_instruction & thumb::ALU_OPERATION_MASK)
                    == thumb::ALU_OPERATION_FORMAT
                {
                    decode_alu_operations(fetched_instruction)
                } else if (fetched_instruction & thumb::MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_MASK)
                    == thumb::MOVE_COMPARE_ADD_SUBTRACT_IMMEDIATE_FORMAT
                {
                    thumb::decode_mcas_immediate(fetched_instruction)
                } else if (fetched_instruction & thumb::ADD_SUBTRACT_MASK)
                    == thumb::ADD_SUBTRACT_FORMAT
                {
                    decode_add_subtract(fetched_instruction)
                } else if (fetched_instruction & thumb::MOVE_SHIFTED_REGISTER_MASK)
                    == thumb::MOVE_SHIFTED_REGISTER_FORMAT
                {
                    decode_move_shifted_register(fetched_instruction)
                } else {
                    return Err(CoreError::OpcodeNotImplemented(fetched_instruction));
                },
            })
        }

        Ok(())
    }

    fn execute(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
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
                Instruction::LongBranchWithLink(d) => d,
                Instruction::HalfwordDataTransfer(d) => d,
            };

            self.log_instruction(
                decoded_instruction.location,
                decoded_instruction.opcode,
                decoded_instruction.condition,
                &ins.mnemonic(),
                &ins.description(&self.registers, bus),
            );

            if self.check_condition(decoded_instruction.condition) {
                let cycles = ins.execute(&mut self.registers, bus);
                if self.registers.pipeline_flush {
                    self.decoded_instruction = None;
                    self.fetched_instruction = None;
                    self.registers.pipeline_flush = false;
                }
                return cycles;
            }
        }

        Ok(1)
    }

    pub fn log_instruction(
        &self,
        address: u32,
        opcode: u32,
        condition: u32,
        mneumonic: &str,
        description: &str,
    ) {
        if self.logging_enabled {
            let condition = Self::get_condition_label(condition);
            println!(
                "${address:08X}: {opcode:08X} {mneumonic}{}{condition} {description}",
                if condition.len() > 0 { "." } else { "" },
            );
        }
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
