use super::{IntCodeMachine, OpCode, Error};
use super::mem::{Fetch, Store, ParamMode};
use super::ops::*;

#[test]
fn test_execute() {
    let program = vec![1, 0, 0, 0, 99];
    let mut machine = IntCodeMachine::boot(program);
    let program = machine.run();
    assert_eq!(program, Ok(vec![2, 0, 0, 0, 99].as_slice()));

    let program = vec![2, 4, 4, 5, 99, 0];
    let mut machine = IntCodeMachine::boot(program);
    let program = machine.run();
    assert_eq!(program, Ok(vec![2, 4, 4, 5, 99, 9801].as_slice()));

    let program = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
    let mut machine = IntCodeMachine::boot(program);
    let program = machine.run();
    assert_eq!(program, Ok(vec![30, 1, 1, 4, 2, 5, 6, 0, 99].as_slice()));
}

#[test]
fn test_get_at() {
    let mut mem = vec![12, 0];
    assert_eq!(mem.fetch(1, &ParamMode::Indirect), Ok(12));
}

#[test]
fn test_get_mut_at() {
    let mut mem = vec![12, 0];
    assert!(mem.store(1, &ParamMode::Indirect, 100).is_ok());
    assert_eq!(mem[0], 100);
}

#[test]
fn test_opcode_execute_add() {
    let mut mem = vec![1, 0, 0, 0];
    let op = Add;
    assert!(op.execute(0, 0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 0, 0, 0]);
}

#[test]
fn test_opcode_execute_multiply() {
    let mut mem = vec![2, 3, 0, 3];
    let op = Multiply;
    assert!(op.execute(0, 0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 3, 0, 6]);
}

#[test]
fn test_decode() {
    let param_code = 0;
    assert_eq!(decode(param_code, 1), Ok(vec![ParamMode::Indirect]));

    let param_code = 1;
    assert_eq!(decode(param_code, 1), Ok(vec![ParamMode::Immediate]));

    let param_code = 2;
    assert_eq!(decode(param_code, 1), Err(Error::InvalidParameter));

    let param_code = 10;
    assert_eq!(decode(param_code, 2), Ok(vec![ParamMode::Indirect, ParamMode::Immediate]));
}
