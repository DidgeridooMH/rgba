#[derive(Copy, Clone)]
enum CpuMode {
    User,
    Fiq,
    Supervisor,
    Abort,
    Irq,
    Undefined,
}

pub struct Cpu {
    reg: [u32; 16],
    fiq_reg: [u32; 7],
    svc_reg: [u32; 2],
    abt_reg: [u32; 2],
    irq_reg: [u32; 2],
    und_reg: [u32; 2],
    mode: CpuMode,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            reg: [0; 16],
            fiq_reg: [0; 7],
            svc_reg: [0; 2],
            abt_reg: [0; 2],
            irq_reg: [0; 2],
            und_reg: [0; 2],
            mode: CpuMode::User,
        }
    }
}

impl Cpu {
    pub fn reg(&mut self, index: usize) -> &mut u32 {
        match self.mode {
            CpuMode::User => &mut self.reg[index],
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

    pub fn tick(&mut self) {}
}
