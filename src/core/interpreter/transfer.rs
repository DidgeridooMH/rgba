use crate::core::{Bus, CoreError};

use super::{CpuMode, Interpreter, OperandType};

pub const SINGLE_TRANSFER_MASK: u32 = 0b0000_1100_0000_0000_0000_0000_0000_0000;
pub const SINGLE_TRANSFER_FORMAT: u32 = 0b0000_0100_0000_0000_0000_0000_0000_0000;

pub const BLOCK_TRANSFER_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BLOCK_TRANSFER_FORMAT: u32 = 0b0000_1000_0000_0000_0000_0000_0000_0000;

impl Interpreter {
    // TODO: R15 storage will store the current instruction plus 12. This is due to pipeling that
    // is not implemented yet.
    //
    // TODO: Big endian is not implemented yet.
    pub fn single_data_transfer(&mut self, opcode: u32, bus: &mut Bus) -> Result<usize, CoreError> {
        let operand_type = if opcode & (1 << 25) > 0 {
            OperandType::Register
        } else {
            OperandType::Immediate
        };

        let operand = match operand_type {
            OperandType::Immediate => opcode & 0xFFF,
            OperandType::Register => self.shift_operand(opcode),
        };

        let base_register_index = (opcode >> 16) & 0xF;
        let mut address = self.reg(base_register_index as usize);

        let pre_index = opcode & (1 << 24) > 0;
        let increment = opcode & (1 << 23) > 0;
        if pre_index {
            if increment {
                address += operand;
            } else {
                address -= operand;
            }
        }

        let load = opcode & (1 << 20) > 0;
        let byte_write = opcode & (1 << 22) > 0;
        let register_index = (opcode >> 12) & 0xF;

        let write_back = opcode & (1 << 21) > 0;
        let mode = if !pre_index && write_back {
            CpuMode::User
        } else {
            self.cpsr.mode
        };
        if load {
            if byte_write {
                *self.reg_with_mode_mut(register_index as usize, mode) =
                    bus.read_byte(address)? as u32;
            } else {
                let mut data = bus.read_dword(address)?;
                if address % 4 == 2 {
                    data = data.rotate_left(16);
                }
                *self.reg_with_mode_mut(register_index as usize, mode) = data;
            }
        } else {
            if byte_write {
                bus.write_byte(address, self.reg(register_index as usize) as u8)?;
            } else {
                bus.write_dword(address, self.reg(register_index as usize))?;
            }
        }

        if !pre_index {
            if increment {
                address += operand;
            } else {
                address -= operand;
            }
        }

        if write_back || !pre_index {
            *self.reg_mut(base_register_index as usize) = address;
        }

        self.log_instruction(
            opcode,
            &format!(
                "{}{}",
                if load { "LDR" } else { "STR" },
                if byte_write { "B" } else { "W" },
            ),
            &format!(
                "r{register_index}({:X}) := r{}, 0x{:X}",
                self.reg(register_index as usize),
                base_register_index,
                operand
            ),
        );

        Ok(1)
    }

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
                if increment { "I" } else { "D" },
                if load { "LDM" } else { "STM" },
                if pre_index { "B" } else { "A" }
            ),
            &format!("r{}, 0x{:X}", base_register_index, opcode & 0xFFFF),
        );

        Ok(1)
    }
}
