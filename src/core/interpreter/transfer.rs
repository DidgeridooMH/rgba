use crate::core::{Bus, CoreError};

use super::{
    instruction::{InstructionExecutor, Operand},
    shift::Shift,
    status::CpuMode,
};

pub const SINGLE_TRANSFER_MASK: u32 = 0b0000_1100_0000_0000_0000_0000_0000_0000;
pub const SINGLE_TRANSFER_FORMAT: u32 = 0b0000_0100_0000_0000_0000_0000_0000_0000;

pub const BLOCK_TRANSFER_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BLOCK_TRANSFER_FORMAT: u32 = 0b0000_1000_0000_0000_0000_0000_0000_0000;

pub const PSR_TRANSFER_MRS_MASK: u32 = 0b0000_1111_1011_1111_0000_0000_0000_0000;
pub const PSR_TRANSFER_MRS_FORMAT: u32 = 0b0000_0001_0000_1111_0000_0000_0000_0000;

pub const PSR_TRANSFER_MSR_MASK: u32 = 0b0000_1101_1011_0000_1111_0000_0000_0000;
pub const PSR_TRANSFER_MSR_FORMAT: u32 = 0b0000_0001_0010_0000_1111_0000_0000_0000;

pub const SINGLE_DATA_SWAP_MASK: u32 = 0b0000_1111_1000_0000_0000_1111_1111_0000;
pub const SINGLE_DATA_SWAP_FORMAT: u32 = 0b0000_0001_0000_0000_0000_0000_1001_0000;

pub struct SingleDataTransferInstruction {
    source_register_index: u32,
    base_register_index: u32,
    offset: Operand,
    load: bool,
    write_back: bool,
    byte_transfer: bool,
    up: bool,
    pre_index: bool,
}

impl SingleDataTransferInstruction {
    pub fn decode(registers: &mut super::register::RegisterBank, opcode: u32) -> Self {
        let offset = if opcode & (1 << 25) > 0 {
            match Shift::from_opcode(opcode) {
                Shift::Immediate(shift) => Operand::Immediate(shift.shift(registers)),
                Shift::Register(shift) => Operand::RegisterShifted(shift),
            }
        } else {
            Operand::Immediate(opcode & 0xFFF)
        };

        Self {
            offset,
            source_register_index: (opcode >> 12) & 0xF,
            base_register_index: (opcode >> 16) & 0xF,
            load: opcode & (1 << 20) > 0,
            write_back: opcode & (1 << 21) > 0,
            byte_transfer: opcode & (1 << 22) > 0,
            up: opcode & (1 << 23) > 0,
            pre_index: opcode & (1 << 24) > 0,
        }
    }
}

impl InstructionExecutor for SingleDataTransferInstruction {
    fn execute(
        &self,
        registers: &mut super::register::RegisterBank,
        bus: &mut Bus,
    ) -> Result<usize, CoreError> {
        let offset = match &self.offset {
            Operand::Immediate(value) => *value,
            Operand::RegisterShifted(shift) => shift.shift(registers),
        };

        let mut address = registers.reg(self.base_register_index as usize);
        if self.pre_index {
            if self.up {
                address += offset;
            } else {
                address -= offset;
            }
        }

        let mode = if !self.pre_index && self.write_back {
            CpuMode::User
        } else {
            registers.cpsr.mode
        };
        if self.load {
            let data = if self.byte_transfer {
                bus.read_byte(address)? as u32
            } else {
                bus.read_dword(address)?
            };
            *registers.reg_with_mode_mut(self.source_register_index as usize, mode) =
                if self.byte_transfer {
                    data & 0xFF
                } else if address % 4 == 2 {
                    data.rotate_left(16)
                } else {
                    data
                };
        } else {
            let mut source_register =
                registers.reg_with_mode(self.source_register_index as usize, mode);
            if self.source_register_index == 15 {
                source_register -= 4;
            }

            if self.byte_transfer {
                bus.write_byte(address, source_register as u8)?;
            } else {
                bus.write_dword(address, source_register)?;
            }
        }

        if !self.pre_index {
            if self.up {
                address += offset;
            } else {
                address -= offset;
            }
        }

        if self.write_back || !self.pre_index {
            *registers.reg_mut(self.base_register_index as usize) = address;
        }

        Ok(1)
    }

    fn mneumonic(&self) -> String {
        format!(
            "{}{}{}",
            if self.load { "ldr" } else { "str" },
            if self.byte_transfer { "b" } else { "" },
            if self.write_back { "t" } else { "" },
        )
    }

    fn description(&self) -> String {
        format!(
            "r{}, [r{}], {}",
            self.source_register_index, self.base_register_index, self.offset
        )
    }
}

/*impl Interpreter {
    pub fn block_data_transfer(&mut self, opcode: u32, bus: &mut Bus) -> Result<usize, CoreError> {
        let pre_index = opcode & (1 << 24) > 0;
        let increment = opcode & (1 << 23) > 0;
        let psr_and_force_user = opcode & (1 << 22) > 0;
        let mut write_back = opcode & (1 << 21) > 0;
        let load = opcode & (1 << 20) > 0;

        let base_register_index = (opcode >> 16) & 0xF;
        let mut base_register = self.reg(base_register_index as usize);

        let mut number_of_address = 0;
        let mut r15_included = false;
        for i in 0..16 {
            if (1 << i) & opcode > 0 {
                if i == 15 {
                    r15_included = true;
                }

                number_of_address += 1;
            }
        }

        let new_address = base_register + number_of_address * 4;

        base_register = if increment {
            new_address
        } else {
            base_register
        };
        if pre_index == increment {
            base_register += 4;
        }

        let register_bank = if (!r15_included || !load) && psr_and_force_user {
            CpuMode::User
        } else {
            self.cpsr.mode
        };
        for i in 0..16 {
            if (1 << i) & opcode > 0 {
                if load {
                    *self.reg_with_mode_mut(i, register_bank) =
                        bus.read_dword(base_register).unwrap();

                    if i == 15 && psr_and_force_user {
                        self.cpsr = self.spsr();
                    }
                } else {
                    bus.write_dword(base_register, self.reg_with_mode(i, register_bank))?;
                }

                base_register += 4;

                if i == base_register_index as usize {
                    write_back = false;
                }

                // Write back's behavior is undefined when using the user mode banks.
                if write_back {
                    *self.reg_mut(base_register_index as usize) = new_address;
                    write_back = false;
                }
            }
        }

        self.log_instruction(
            opcode,
            &format!(
                "{}{}{}",
                if load { "ldm" } else { "stm" },
                if increment { "i" } else { "d" },
                if pre_index { "b" } else { "a" }
            ),
            &format!("r{}, #0x{:X}", base_register_index, opcode & 0xFFFF),
        );

        Ok(1)
    }

    /// Transfer PSR contents to a register.
    pub fn psr_transfer_mrs(&mut self, opcode: u32) -> usize {
        let use_spsr = opcode & (1 << 22) > 0;
        let psr = if use_spsr { self.spsr() } else { self.cpsr };
        let destination_register_index = (opcode >> 12) & 0xF;

        let psr = psr.to_u32();
        *self.reg_mut(destination_register_index as usize) = psr;

        self.log_instruction(
            opcode,
            "mrs",
            &format!(
                "r{destination_register_index} := {}{:X}",
                if use_spsr { "spsr" } else { "cpsr" },
                psr
            ),
        );

        1
    }

    /// Transfer register contents or immediate to PSR.
    pub fn psr_transfer_msr(&mut self, opcode: u32) -> usize {
        let operand_type = if opcode & (1 << 25) > 0 {
            OperandType::Immediate
        } else {
            OperandType::Register
        };
        let use_spsr = opcode & (1 << 22) > 0;

        let operand = match operand_type {
            OperandType::Immediate => Self::shift_immediate(opcode),
            OperandType::Register => self.reg((opcode & 0xF) as usize),
        };

        let psr = if use_spsr {
            self.spsr_mut()
        } else {
            &mut self.cpsr
        };

        let psr_operand = ProgramStatusRegister::from_u32(operand);
        let write_flags = opcode & (1 << 19) > 0;
        if write_flags {
            (*psr).zero = psr_operand.zero;
            (*psr).signed = psr_operand.signed;
            (*psr).carry = psr_operand.carry;
            (*psr).overflow = psr_operand.overflow;
        }

        let write_control = opcode & (1 << 16) > 0;
        if write_control {
            (*psr).irq_disable = psr_operand.irq_disable;
            (*psr).fiq_disable = psr_operand.fiq_disable;
            (*psr).instruction_mode = psr_operand.instruction_mode;
            (*psr).mode = psr_operand.mode;
        }

        self.log_instruction(
            opcode,
            "msr",
            &format!(
                "{}_{}{}, 0x{operand:X}",
                if use_spsr { "spsr" } else { "cpsr" },
                if write_flags { "f" } else { "" },
                if write_control { "c" } else { "" }
            ),
        );

        1
    }

    // TODO: Under the hood swp is implemented as a ldr and str instruction.
    pub fn single_data_swap(&mut self, opcode: u32, bus: &mut Bus) -> Result<usize, CoreError> {
        let source_register_index = opcode & 0xF;
        let destination_register_index = (opcode >> 12) & 0xF;
        let base_register_index = (opcode >> 16) & 0xF;

        let address = self.reg(base_register_index as usize);

        let data = bus.read_dword(address)?;
        let source_register = self.reg(source_register_index as usize);

        let byte_transfer = opcode & (1 << 22) > 0;
        if byte_transfer {
            bus.write_byte(address, source_register as u8)?;
            *self.reg_mut(destination_register_index as usize) = data & 0xFF;
        } else {
            bus.write_dword(address, source_register)?;
            *self.reg_mut(destination_register_index as usize) = data;
        }

        self.log_instruction(
            opcode,
            if byte_transfer { "swpb" } else { "swp" },
            &format!(
                "r{destination_register_index}, r{source_register_index}, [r{base_register_index}]",
                destination_register_index = destination_register_index,
                base_register_index = base_register_index,
                source_register_index = source_register_index
            ),
        );

        Ok(1)
    }
}*/
