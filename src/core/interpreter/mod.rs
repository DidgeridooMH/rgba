mod arithmetic;
mod branch;
mod interrupt;
mod shift;
mod transfer;

use self::{
    arithmetic::{DATA_PROCESSING_FORMAT, DATA_PROCESSING_MASK},
    branch::{BRANCH_AND_EXCHANGE_FORMAT, BRANCH_AND_EXCHANGE_MASK, BRANCH_FORMAT, BRANCH_MASK},
    interrupt::{SOFTWARE_INTERRUPT_FORMAT, SOFTWARE_INTERRUPT_MASK},
    transfer::{
        BLOCK_TRANSFER_FORMAT, BLOCK_TRANSFER_MASK, SINGLE_TRANSFER_FORMAT, SINGLE_TRANSFER_MASK,
    },
};

use super::{Bus, CoreError};

#[derive(Copy, Clone, Default)]
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

#[derive(Copy, Clone, Default)]
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

    instruction_mode: InstructionMode,
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
        } else if (opcode & DATA_PROCESSING_MASK) == DATA_PROCESSING_FORMAT {
            Ok(self.process_data(opcode))
        } else {
            Err(CoreError::OpcodeNotImplemented(opcode))
        }
    }

    pub fn log_instruction(&self, opcode: u32, mneumonic: &str, description: &str) {
        println!(
            "${:08X}: {:08X} {mneumonic}{} {description}",
            self.pc() - 4,
            opcode,
            Self::get_condition_label(opcode >> 28)
        );
    }

    fn get_condition_label(condition_code: u32) -> &'static str {
        match condition_code {
            0x0 => "EQ",
            0x1 => "NE",
            0x2 => "CS",
            0x3 => "CC",
            0x4 => "MI",
            0x5 => "PL",
            0x6 => "VS",
            0x7 => "VC",
            0x8 => "HI",
            0x9 => "LS",
            0xA => "GE",
            0xB => "LT",
            0xC => "GT",
            0xD => "LE",
            0xE => "",
            0xF => "NV",
            _ => unreachable!(),
        }
    }

    fn check_condition(&self, condition: u32) -> bool {
        match condition {
            0x0 => self.cpsr.zero,
            _ => true
        }
    }
}
