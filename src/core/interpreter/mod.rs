mod arithmetic;
mod branch;
mod interrupt;
mod shift;
mod transfer;
mod multiply;

use transfer::{
    PSR_TRANSFER_MRS_FORMAT, PSR_TRANSFER_MRS_MASK, PSR_TRANSFER_MSR_FORMAT, PSR_TRANSFER_MSR_MASK,
};

use self::{
    arithmetic::{DATA_PROCESSING_FORMAT, DATA_PROCESSING_MASK}, branch::{BRANCH_AND_EXCHANGE_FORMAT, BRANCH_AND_EXCHANGE_MASK, BRANCH_FORMAT, BRANCH_MASK}, interrupt::{SOFTWARE_INTERRUPT_FORMAT, SOFTWARE_INTERRUPT_MASK}, multiply::{MULTIPLY_FORMAT, MULTIPLY_LONG_FORMAT, MULTIPLY_MASK}, transfer::{
        BLOCK_TRANSFER_FORMAT, BLOCK_TRANSFER_MASK, SINGLE_DATA_SWAP_FORMAT, SINGLE_DATA_SWAP_MASK, SINGLE_TRANSFER_FORMAT, SINGLE_TRANSFER_MASK
    }
};

use super::{Bus, CoreError};

#[derive(Copy, Clone, Default)]
#[repr(u32)]
enum CpuMode {
    #[default]
    User = 0x10,
    Fiq = 0x11,
    Irq = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1b,
    System = 0x1f,
}

impl CpuMode {
    pub fn from_u32(mode: u32) -> CpuMode {
        match mode {
            0x10 => CpuMode::User,
            0x11 => CpuMode::Fiq,
            0x12 => CpuMode::Irq,
            0x13 => CpuMode::Supervisor,
            0x17 => CpuMode::Abort,
            0x1F => CpuMode::System,
            _ => CpuMode::Undefined,
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(u32)]
enum InstructionMode {
    #[default]
    Arm = 0,
    Thumb = 1,
}

#[derive(Copy, Clone, Default)]
pub struct ProgramStatusRegister {
    signed: bool,
    zero: bool,
    carry: bool,
    overflow: bool,
    sticky_overflow: bool,
    irq_disable: bool,
    fiq_disable: bool,
    instruction_mode: InstructionMode,
    mode: CpuMode,
}

impl ProgramStatusRegister {
    pub fn to_u32(&self) -> u32 {
        ((self.signed as u32) << 31)
            | ((self.zero as u32) << 30)
            | ((self.carry as u32) << 29)
            | ((self.overflow as u32) << 28)
            | ((self.sticky_overflow as u32) << 27)
            | ((self.irq_disable as u32) << 7)
            | ((self.fiq_disable as u32) << 6)
            | ((self.instruction_mode as u32) << 5)
            | self.mode as u32
    }

    pub fn from_u32(psr: u32) -> Self {
        Self {
            signed: psr & (1 << 31) > 0,
            zero: psr & (1 << 30) > 0,
            carry: psr & (1 << 29) > 0,
            overflow: psr & (1 << 28) > 0,
            sticky_overflow: psr & (1 << 27) > 0,
            irq_disable: psr & (1 << 7) > 0,
            fiq_disable: psr & (1 << 6) > 0,
            instruction_mode: if psr & (1 << 5) > 0 {
                InstructionMode::Thumb
            } else {
                InstructionMode::Arm
            },
            mode: CpuMode::from_u32(psr & 0x1F),
        }
    }
}

enum OperandType {
    Immediate,
    Register,
}

#[derive(Default)]
pub struct Interpreter {
    reg: [u32; 16],
    fiq_reg: [u32; 7],
    svc_reg: [u32; 2],
    abt_reg: [u32; 2],
    irq_reg: [u32; 2],
    und_reg: [u32; 2],
    spsr: [ProgramStatusRegister; 5],
    cpsr: ProgramStatusRegister,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self::default();

        interpreter.spsr[0].mode = CpuMode::Fiq;
        interpreter.spsr[1].mode = CpuMode::Supervisor;
        interpreter.spsr[2].mode = CpuMode::Irq;
        interpreter.spsr[3].mode = CpuMode::Abort;
        interpreter.spsr[4].mode = CpuMode::Undefined;

        interpreter
    }

    pub fn pc_mut(&mut self) -> &mut u32 {
        self.reg_mut(15)
    }

    pub fn pc(&self) -> u32 {
        self.reg(15)
    }

    fn spsr_with_mode_mut(&mut self, mode: CpuMode) -> &mut ProgramStatusRegister {
        match mode {
            CpuMode::Fiq => &mut self.spsr[0],
            CpuMode::Supervisor => &mut self.spsr[1],
            CpuMode::Irq => &mut self.spsr[2],
            CpuMode::Abort => &mut self.spsr[3],
            CpuMode::Undefined => &mut self.spsr[4],
            _ => {
                println!("Warning: SPSR is not defined for supervisor mode.");
                &mut self.spsr[0]
            }
        }
    }

    fn spsr_with_mode(&mut self, mode: CpuMode) -> ProgramStatusRegister {
        match mode {
            CpuMode::Fiq => self.spsr[0],
            CpuMode::Supervisor => self.spsr[1],
            CpuMode::Irq => self.spsr[2],
            CpuMode::Abort => self.spsr[3],
            CpuMode::Undefined => self.spsr[4],
            _ => {
                println!("Warning: SPSR is not defined for supervisor mode.");
                self.spsr[0]
            }
        }
    }

    pub fn spsr_mut(&mut self) -> &mut ProgramStatusRegister {
        self.spsr_with_mode_mut(self.cpsr.mode)
    }

    pub fn spsr(&mut self) -> ProgramStatusRegister {
        self.spsr_with_mode(self.cpsr.mode)
    }

    fn reg_with_mode_mut(&mut self, index: usize, mode: CpuMode) -> &mut u32 {
        match mode {
            CpuMode::User | CpuMode::System => &mut self.reg[index],
            CpuMode::Fiq => {
                if index < 8 || index == 15 {
                    &mut self.reg[index]
                } else {
                    &mut self.fiq_reg[index - 7]
                }
            }
            CpuMode::Supervisor => {
                if index != 13 && index != 14 {
                    &mut self.reg[index]
                } else {
                    &mut self.svc_reg[index - 13]
                }
            }
            CpuMode::Irq => {
                if index != 13 && index != 14 {
                    &mut self.reg[index]
                } else {
                    &mut self.irq_reg[index - 13]
                }
            }
            CpuMode::Abort => {
                if index != 13 && index != 14 {
                    &mut self.reg[index]
                } else {
                    &mut self.abt_reg[index - 13]
                }
            }
            CpuMode::Undefined => {
                if index != 13 && index != 14 {
                    &mut self.reg[index]
                } else {
                    &mut self.und_reg[index - 13]
                }
            }
        }
    }

    fn reg_with_mode(&self, index: usize, mode: CpuMode) -> u32 {
        match mode {
            CpuMode::User | CpuMode::System => self.reg[index],
            CpuMode::Fiq => {
                if index < 8 || index == 15 {
                    self.reg[index]
                } else {
                    self.fiq_reg[index - 7]
                }
            }
            CpuMode::Supervisor => {
                if index != 13 && index != 14 {
                    self.reg[index]
                } else {
                    self.svc_reg[index - 13]
                }
            }
            CpuMode::Irq => {
                if index != 13 && index != 14 {
                    self.reg[index]
                } else {
                    self.irq_reg[index - 13]
                }
            }
            CpuMode::Abort => {
                if index != 13 && index != 14 {
                    self.reg[index]
                } else {
                    self.abt_reg[index - 13]
                }
            }
            CpuMode::Undefined => {
                if index != 13 && index != 14 {
                    self.reg[index]
                } else {
                    self.und_reg[index - 13]
                }
            }
        }
    }

    pub fn reg_mut(&mut self, index: usize) -> &mut u32 {
        self.reg_with_mode_mut(index, self.cpsr.mode)
    }

    pub fn reg(&self, index: usize) -> u32 {
        self.reg_with_mode(index, self.cpsr.mode)
    }

    // TODO: Implement pipelining?
    pub fn tick(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        match self.cpsr.instruction_mode {
            InstructionMode::Arm => self.tick_arm(bus),
            InstructionMode::Thumb => unimplemented!(),
        }
    }

    pub fn tick_arm(&mut self, bus: &mut Bus) -> Result<usize, CoreError> {
        let opcode = bus.read_dword(*self.pc_mut()).unwrap();

        *self.pc_mut() += 4;

        if !self.check_condition(opcode >> 28) {
            println!("${:08X}: {:08X} SKIPPED", self.pc() - 4, opcode);
            return Ok(1);
        }

        if (opcode & BRANCH_AND_EXCHANGE_MASK) == BRANCH_AND_EXCHANGE_FORMAT {
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
        }
    }

    pub fn log_instruction(&self, opcode: u32, mneumonic: &str, description: &str) {
        let condition = Self::get_condition_label(opcode >> 28);
        println!(
            "${:08X}: {opcode:08X} {mneumonic}{}{condition} {description}",
            self.pc() - 4,
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
            0x0 => self.cpsr.zero,
            0x1 => !self.cpsr.zero,
            0x2 => self.cpsr.carry,
            0x3 => !self.cpsr.carry,
            0x4 => self.cpsr.signed,
            0x5 => !self.cpsr.signed,
            0x6 => self.cpsr.overflow,
            0x7 => !self.cpsr.overflow,
            0x8 => self.cpsr.carry && !self.cpsr.zero,
            0x9 => !self.cpsr.carry || self.cpsr.zero,
            0xA => self.cpsr.signed == self.cpsr.overflow,
            0xB => self.cpsr.signed != self.cpsr.overflow,
            0xC => !self.cpsr.zero && self.cpsr.signed == self.cpsr.overflow,
            0xD => self.cpsr.zero || self.cpsr.signed != self.cpsr.overflow,
            0xE => true,
            0xF => false,
            _ => true,
        }
    }
}
