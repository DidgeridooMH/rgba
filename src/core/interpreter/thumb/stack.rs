use crate::core::interpreter::{
    arm::{BlockDataTransferInstruction, DataProcessingInstruction, DataProcessingOperation},
    instruction::{Instruction, Operand},
};

pub const PUSH_POP_REGISTERS_FORMAT: u32 = 0b1011_0100_0000_0000;
pub const PUSH_POP_REGISTERS_MASK: u32 = 0b1111_0110_0000_0000;

pub const ADD_OFFSET_TO_STACK_POINTER_FORMAT: u32 = 0b1011_0000_0000_0000;
pub const ADD_OFFSET_TO_STACK_POINTER_MASK: u32 = 0b1111_1111_0000_0000;

pub fn decode_push_pop_registers(opcode: u32) -> Instruction {
    let mut register_list = opcode & 0x00FF;
    let mut number_of_registers = 0;
    for i in 0..8 {
        if (register_list >> i) & 1 > 0 {
            number_of_registers += 1;
        }
    }
    let load = (opcode >> 11) & 1 > 0;

    let store_lr = (opcode >> 8) & 1 > 0;
    if store_lr {
        register_list |= 1 << if load { 15 } else { 14 };
        number_of_registers += 1;
    }

    Instruction::BlockDataTransfer(BlockDataTransferInstruction::new(
        13,
        register_list as u16,
        load,
        true,
        load,
        !load,
        false,
        number_of_registers,
    ))
}

pub fn decode_add_offset_stack_pointer(opcode: u32) -> Instruction {
    let offset = (opcode & 0x7F) << 2;
    let sign = (opcode >> 7) & 1 > 0;

    Instruction::DataProcessing(DataProcessingInstruction::new(
        false,
        13,
        Operand::Immediate(offset),
        Some(13),
        if sign {
            DataProcessingOperation::Subtract
        } else {
            DataProcessingOperation::Add
        },
    ))
}
