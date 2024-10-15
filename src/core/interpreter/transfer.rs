use crate::core::{Bus, CoreError};

use super::{CpuMode, Interpreter};

pub const BLOCK_TRANSFER_MASK: u32 = 0b0000_1110_0000_0000_0000_0000_0000_0000;
pub const BLOCK_TRANSFER_FORMAT: u32 = 0b0000_1000_0000_0000_0000_0000_0000_0000;

impl Interpreter {
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

        Ok(1)
    }
}
