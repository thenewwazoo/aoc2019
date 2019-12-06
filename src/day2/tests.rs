use super::{IntCodeMachine, OpCode, IndirectFetch, IndirectStore};
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
    assert_eq!(mem.get_at(1), Ok(12));
}

#[test]
fn test_get_mut_at() {
    let mut mem = vec![12, 0];
    assert!(mem.store_at(1, 100).is_ok());
    assert_eq!(mem[0], 100);
}

#[test]
fn test_opcode_execute_add() {
    let mut mem = vec![1, 0, 0, 0];
    let op = Add;
    assert!(op.execute(0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 0, 0, 0]);
}

#[test]
fn test_opcode_execute_multiply() {
    let mut mem = vec![2, 3, 0, 3];
    let op = Multiply;
    assert!(op.execute(0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 3, 0, 6]);
}
