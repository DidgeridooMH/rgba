use crate::core::interpreter::status::ProgramStatusRegister;
use crate::core::{Bus, CoreError};

use crate::core::interpreter::{
    instruction::{InstructionExecutor, Operand},
    register::RegisterBank,
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
    force_word_alignment: bool,
}

impl SingleDataTransferInstruction {
    pub fn new(
        source_register_index: u32,
        base_register_index: u32,
        offset: Operand,
        load: bool,
        write_back: bool,
        byte_transfer: bool,
        up: bool,
        pre_index: bool,
        force_word_alignment: bool,
    ) -> Self {
        Self {
            source_register_index,
            base_register_index,
            offset,
            load,
            write_back,
            byte_transfer,
            up,
            pre_index,
            force_word_alignment,
        }
    }

    pub fn decode(registers: &mut RegisterBank, opcode: u32) -> Self {
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
            force_word_alignment: false,
        }
    }

    fn calculate_address(&self, registers: &RegisterBank) -> u32 {
        let mut address = registers.reg(self.base_register_index as usize);

        if self.force_word_alignment {
            address &= !0b10;
        }

        if self.pre_index {
            self.offset_address(address, registers)
        } else {
            address
        }
    }

    fn offset_address(&self, address: u32, registers: &RegisterBank) -> u32 {
        let offset = self.offset.value(registers);
        if self.up {
            address.wrapping_add(offset)
        } else {
            address.wrapping_sub(offset)
        }
    }
}

impl InstructionExecutor for SingleDataTransferInstruction {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError> {
        let address = self.calculate_address(registers);

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
                if !self.byte_transfer && address % 4 == 2 {
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

        if self.write_back || !self.pre_index {
            *registers.reg_mut(self.base_register_index as usize) = if !self.pre_index {
                self.offset_address(address, registers)
            } else {
                address
            }
        }

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        format!(
            "{}{}{}",
            if self.load { "ldr" } else { "str" },
            if self.byte_transfer { "b" } else { "" },
            if self.write_back { "t" } else { "" },
        )
    }

    fn description(&self, registers: &RegisterBank, bus: &mut Bus) -> String {
        let address_hint = if self.load {
            let address = self.calculate_address(registers);
            let data = if self.byte_transfer {
                match bus.read_byte(address) {
                    Ok(data) => Ok(data as u32),
                    Err(_) => Err(()),
                }
            } else {
                match bus.read_dword(address) {
                    Ok(data) => Ok(data),
                    Err(_) => Err(()),
                }
            };

            let data = match data {
                Ok(d) => format!(
                    "${:X}",
                    if !self.byte_transfer && address % 4 == 2 {
                        d.rotate_left(16)
                    } else {
                        d
                    }
                ),
                Err(_) => "???".to_string(),
            };
            format!("(={})", data)
        } else {
            "".into()
        };
        format!(
            "r{}, [r{}], {} {address_hint}",
            self.source_register_index, self.base_register_index, self.offset
        )
    }
}

pub struct BlockDataTransferInstruction {
    base_register_index: u32,
    registers: u16,
    load: bool,
    write_back: bool,
    increment: bool,
    pre_index: bool,
    psr_and_force_user: bool,
    number_of_registers: u32,
}

impl BlockDataTransferInstruction {
    pub fn new(
        base_register_index: u32,
        registers: u16,
        load: bool,
        write_back: bool,
        increment: bool,
        pre_index: bool,
        psr_and_force_user: bool,
        number_of_registers: u32,
    ) -> Self {
        Self {
            base_register_index,
            registers,
            load,
            write_back,
            increment,
            pre_index,
            psr_and_force_user,
            number_of_registers,
        }
    }

    pub fn decode(_registers: &mut RegisterBank, opcode: u32) -> Self {
        let base_register_index = (opcode >> 16) & 0xF;
        let mut number_of_registers = 0;
        for i in 0..16 {
            if (1 << i) & opcode > 0 {
                number_of_registers += 1;
            }
        }

        Self {
            base_register_index,
            registers: (opcode & 0xFFFF) as u16,
            number_of_registers,
            load: opcode & (1 << 20) > 0,
            write_back: opcode & (1 << 21) > 0,
            increment: opcode & (1 << 23) > 0,
            pre_index: opcode & (1 << 24) > 0,
            psr_and_force_user: opcode & (1 << 22) > 0,
        }
    }
}

impl InstructionExecutor for BlockDataTransferInstruction {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError> {
        let base_register = registers.reg(self.base_register_index as usize);
        let mut base_address = if self.increment {
            base_register
        } else {
            base_register - 4 * (self.number_of_registers - 1)
        };

        let register_bank =
            if (((self.registers & (1 << 15)) == 0) || !self.load) && self.psr_and_force_user {
                CpuMode::User
            } else {
                registers.cpsr.mode
            };

        let new_address = if self.increment {
            base_address + 4 * self.number_of_registers as u32
        } else {
            base_address
        };
        for i in 0..16 {
            if (1 << i) & self.registers > 0 {
                if self.pre_index {
                    base_address += 4;
                }

                if self.load {
                    *registers.reg_with_mode_mut(i as usize, register_bank) =
                        bus.read_dword(base_register).unwrap();

                    if i == 15 && self.psr_and_force_user {
                        registers.cpsr = registers.spsr();
                    }
                } else {
                    bus.write_dword(
                        base_register,
                        registers.reg_with_mode(i as usize, register_bank),
                    )?;
                }

                if !self.pre_index {
                    base_address += 4;
                }

                // Write back's behavior is undefined when using the user mode banks.
                if self.write_back {
                    *registers.reg_mut(self.base_register_index as usize) = new_address;
                }
            }
        }

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        format!(
            "{}{}{}",
            if self.load { "ldm" } else { "stm" },
            if self.increment { "i" } else { "d" },
            if self.pre_index { "b" } else { "a" }
        )
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        let mut desc = format!("r{}", self.base_register_index);
        if self.write_back {
            desc.push_str("!");
        }

        desc.push_str(", {");
        let mut first = true;
        for i in 0..16 {
            if (1 << i) & self.registers > 0 {
                if !first {
                    desc.push_str(", ");
                }
                first = false;
                desc.push_str(&format!("r{}", i));
            }
        }

        desc.push_str("}");

        if self.psr_and_force_user {
            desc.push_str("^");
        }

        desc
    }
}

pub struct PsrTransferMrsInstruction {
    destination_register_index: u32,
    use_spsr: bool,
}

impl PsrTransferMrsInstruction {
    pub fn decode(opcode: u32) -> Self {
        Self {
            destination_register_index: (opcode >> 12) & 0xF,
            use_spsr: opcode & (1 << 22) > 0,
        }
    }
}

impl InstructionExecutor for PsrTransferMrsInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        let psr = if self.use_spsr {
            registers.spsr().to_u32()
        } else {
            registers.cpsr.to_u32()
        };

        *registers.reg_mut(self.destination_register_index as usize) = psr;

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        "mrs".to_string()
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        format!(
            "r{}, {}",
            self.destination_register_index,
            if self.use_spsr { "spsr" } else { "cpsr" },
        )
    }
}

pub struct PsrTransferMsrInstruction {
    operand: Operand,
    use_spsr: bool,
    write_flags: bool,
    write_control: bool,
}

impl PsrTransferMsrInstruction {
    pub fn decode(registers: &mut RegisterBank, opcode: u32) -> Self {
        let operand = if opcode & (1 << 25) > 0 {
            Operand::Immediate(opcode & 0xFFF)
        } else {
            match Shift::from_opcode(opcode) {
                Shift::Immediate(shift) => Operand::Immediate(shift.shift(registers)),
                Shift::Register(shift) => Operand::RegisterShifted(shift),
            }
        };

        Self {
            operand,
            use_spsr: opcode & (1 << 22) > 0,
            write_flags: opcode & (1 << 19) > 0,
            write_control: opcode & (1 << 16) > 0,
        }
    }
}

impl InstructionExecutor for PsrTransferMsrInstruction {
    fn execute(&self, registers: &mut RegisterBank, _bus: &mut Bus) -> Result<usize, CoreError> {
        let operand = self.operand.value(registers);

        let psr = if self.use_spsr {
            registers.spsr_mut()
        } else {
            &mut registers.cpsr
        };

        let psr_operand = ProgramStatusRegister::from_u32(operand);
        if self.write_flags {
            psr.zero = psr_operand.zero;
            psr.signed = psr_operand.signed;
            psr.carry = psr_operand.carry;
            psr.overflow = psr_operand.overflow;
        }

        if self.write_control {
            psr.irq_disable = psr_operand.irq_disable;
            psr.fiq_disable = psr_operand.fiq_disable;
            psr.instruction_mode = psr_operand.instruction_mode;
            psr.mode = psr_operand.mode;
        }

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        "msr".to_string()
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        format!(
            "{}_{}{}, {}",
            if self.use_spsr { "spsr" } else { "cpsr" },
            if self.write_flags { "f" } else { "" },
            if self.write_control { "c" } else { "" },
            self.operand
        )
    }
}

pub struct SingleDataSwapInstruction {
    source_register_index: u32,
    destination_register_index: u32,
    base_register_index: u32,
    byte_transfer: bool,
}

impl SingleDataSwapInstruction {
    pub fn decode(opcode: u32) -> Self {
        Self {
            source_register_index: opcode & 0xF,
            destination_register_index: (opcode >> 12) & 0xF,
            base_register_index: (opcode >> 16) & 0xF,
            byte_transfer: opcode & (1 << 22) > 0,
        }
    }
}

impl InstructionExecutor for SingleDataSwapInstruction {
    fn execute(&self, registers: &mut RegisterBank, bus: &mut Bus) -> Result<usize, CoreError> {
        let address = registers.reg(self.base_register_index as usize);
        let data = if self.byte_transfer {
            bus.read_byte(address)? as u32
        } else {
            bus.read_dword(address)?
        };

        let source_register = registers.reg(self.source_register_index as usize);

        if self.byte_transfer {
            bus.write_byte(address, source_register as u8)?;
            *registers.reg_mut(self.destination_register_index as usize) = data & 0xFF;
        } else {
            bus.write_dword(address, source_register)?;
            *registers.reg_mut(self.destination_register_index as usize) = data;
        }

        Ok(1)
    }

    fn mnemonic(&self) -> String {
        format!("swp{}", if self.byte_transfer { "b" } else { "" })
    }

    fn description(&self, _registers: &RegisterBank, _bus: &mut Bus) -> String {
        format!(
            "r{}, r{}, [r{}]",
            self.destination_register_index, self.source_register_index, self.base_register_index
        )
    }
}
