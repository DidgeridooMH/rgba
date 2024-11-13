use std::{cell::RefCell, rc::Rc};

use crate::core::{
    interpreter::{
        arm::BlockDataTransferInstruction, instruction::InstructionExecutor, register::RegisterBank,
    },
    memory::wram::Wram,
    Bus, CoreError,
};

fn setup() -> (Bus, RegisterBank) {
    let wram = Wram::new(0, 1024);

    let mut bus = Bus::default();
    bus.register_region(0..=1023, Rc::new(RefCell::new(wram)));

    (bus, RegisterBank::default())
}

#[test]
fn stmia() -> Result<(), CoreError> {
    const EXPECTED_RESULT: [u32; 4] = [10, 20, 30, 40];

    let (mut bus, mut registers) = setup();

    for i in 0..(EXPECTED_RESULT.len()) {
        *registers.reg_mut(i) = EXPECTED_RESULT[i];
    }
    *registers.reg_mut(13) = 0;

    let instruction =
        BlockDataTransferInstruction::new(13, 0b1111, false, true, true, false, false, 4);

    let _ = instruction.execute(&mut registers, &mut bus);

    let result = [
        bus.read_dword(0)?,
        bus.read_dword(4)?,
        bus.read_dword(8)?,
        bus.read_dword(12)?,
    ];

    assert_eq!(result[0], EXPECTED_RESULT[0]);
    assert_eq!(result[1], EXPECTED_RESULT[1]);
    assert_eq!(result[2], EXPECTED_RESULT[2]);
    assert_eq!(result[3], EXPECTED_RESULT[3]);

    assert_eq!(registers.reg(13), 16);

    Ok(())
}

#[test]
fn stmib() -> Result<(), CoreError> {
    const EXPECTED_RESULT: [u32; 4] = [10, 20, 30, 40];

    let (mut bus, mut registers) = setup();

    for i in 0..(EXPECTED_RESULT.len()) {
        *registers.reg_mut(i) = EXPECTED_RESULT[i];
    }
    *registers.reg_mut(13) = 0;

    let instruction =
        BlockDataTransferInstruction::new(13, 0b1111, false, true, true, true, false, 4);

    let _ = instruction.execute(&mut registers, &mut bus);

    let result = [
        bus.read_dword(4)?,
        bus.read_dword(8)?,
        bus.read_dword(12)?,
        bus.read_dword(16)?,
    ];

    assert_eq!(result[0], EXPECTED_RESULT[0]);
    assert_eq!(result[1], EXPECTED_RESULT[1]);
    assert_eq!(result[2], EXPECTED_RESULT[2]);
    assert_eq!(result[3], EXPECTED_RESULT[3]);

    assert_eq!(registers.reg(13), 16);

    Ok(())
}

#[test]
fn stmda() -> Result<(), CoreError> {
    const EXPECTED_RESULT: [u32; 4] = [10, 20, 30, 40];

    let (mut bus, mut registers) = setup();

    for i in 0..(EXPECTED_RESULT.len()) {
        *registers.reg_mut(i) = EXPECTED_RESULT[i];
    }
    *registers.reg_mut(13) = 12;

    let instruction =
        BlockDataTransferInstruction::new(13, 0b1111, false, true, false, false, false, 4);

    let _ = instruction.execute(&mut registers, &mut bus);

    let result = [
        bus.read_dword(0)?,
        bus.read_dword(4)?,
        bus.read_dword(8)?,
        bus.read_dword(12)?,
    ];

    assert_eq!(result[0], EXPECTED_RESULT[0]);
    assert_eq!(result[1], EXPECTED_RESULT[1]);
    assert_eq!(result[2], EXPECTED_RESULT[2]);
    assert_eq!(result[3], EXPECTED_RESULT[3]);

    assert_eq!(registers.reg(13), 0);

    Ok(())
}

#[test]
fn stmdb() -> Result<(), CoreError> {
    const EXPECTED_RESULT: [u32; 4] = [10, 20, 30, 40];

    let (mut bus, mut registers) = setup();

    for i in 0..(EXPECTED_RESULT.len()) {
        *registers.reg_mut(i) = EXPECTED_RESULT[i];
    }
    *registers.reg_mut(13) = 16;

    let instruction =
        BlockDataTransferInstruction::new(13, 0b1111, false, true, false, true, false, 4);

    let _ = instruction.execute(&mut registers, &mut bus);

    let result = [
        bus.read_dword(0)?,
        bus.read_dword(4)?,
        bus.read_dword(8)?,
        bus.read_dword(12)?,
    ];

    assert_eq!(result[0], EXPECTED_RESULT[0]);
    assert_eq!(result[1], EXPECTED_RESULT[1]);
    assert_eq!(result[2], EXPECTED_RESULT[2]);
    assert_eq!(result[3], EXPECTED_RESULT[3]);

    assert_eq!(registers.reg(13), 0);

    Ok(())
}
