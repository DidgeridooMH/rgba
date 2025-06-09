#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum CpuMode {
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

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum InstructionMode {
    #[default]
    Arm = 0,
    Thumb = 1,
}

#[derive(Copy, Clone, Default)]
pub struct ProgramStatusRegister {
    pub signed: bool,
    pub zero: bool,
    pub carry: bool,
    pub overflow: bool,
    pub sticky_overflow: bool,
    pub irq_disable: bool,
    pub fiq_disable: bool,
    pub instruction_mode: InstructionMode,
    pub mode: CpuMode,
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
