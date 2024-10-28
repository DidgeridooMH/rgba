use super::status::{CpuMode, ProgramStatusRegister};

pub struct RegisterBank {
    reg: [u32; 16],
    fiq_reg: [u32; 7],
    svc_reg: [u32; 2],
    abt_reg: [u32; 2],
    irq_reg: [u32; 2],
    und_reg: [u32; 2],
    spsr: [ProgramStatusRegister; 5],
    pub cpsr: ProgramStatusRegister,
}

impl Default for RegisterBank {
    fn default() -> Self {
        let mut s = Self {
            reg: [0; 16],
            fiq_reg: [0; 7],
            svc_reg: [0; 2],
            abt_reg: [0; 2],
            irq_reg: [0; 2],
            und_reg: [0; 2],
            spsr: [ProgramStatusRegister::default(); 5],
            cpsr: ProgramStatusRegister::default(),
        };

        s.spsr[0].mode = CpuMode::Fiq;
        s.spsr[1].mode = CpuMode::Supervisor;
        s.spsr[2].mode = CpuMode::Irq;
        s.spsr[3].mode = CpuMode::Abort;
        s.spsr[4].mode = CpuMode::Undefined;

        s
    }
}

impl RegisterBank {
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

    pub fn reg_with_mode_mut(&mut self, index: usize, mode: CpuMode) -> &mut u32 {
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

    pub fn reg_with_mode(&self, index: usize, mode: CpuMode) -> u32 {
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
}
