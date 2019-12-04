use super::execute;
use super::OpCode;

#[test]
fn test_execute() {
    let mut program = vec![1, 0, 0, 0, 99];
    assert_eq!(execute(&mut program), Ok(()));
    assert_eq!(program, vec![2, 0, 0, 0, 99]);
    let mut program = vec![2, 3, 0, 3, 99];
    assert_eq!(execute(&mut program), Ok(()));
    assert_eq!(program, vec![2, 3, 0, 6, 99]);
    let mut program = vec![2, 4, 4, 5, 99, 0];
    assert_eq!(execute(&mut program), Ok(()));
    assert_eq!(program, vec![2, 4, 4, 5, 99, 9801]);
    let mut program = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
    assert_eq!(execute(&mut program), Ok(()));
    assert_eq!(program, vec![30, 1, 1, 4, 2, 5, 6, 0, 99]);
}

#[test]
fn test_get_at() {
    let mut mem = vec![12, 0];
    assert_eq!(OpCode::get_at(1, &mut mem), Ok(12));
}

#[test]
fn test_get_mut_at() {
    let mut mem = vec![12, 0];
    let r = OpCode::get_mut_at(1, &mut mem);
    assert_eq!(r, Ok(&mut 12));
    *r.unwrap() = 100;
    assert_eq!(mem[0], 100);
}

#[test]
fn test_opcode_execute_add() {
    let mut mem = vec![1, 0, 0, 0];
    let op = OpCode::Add;
    assert!(op.execute(0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 0, 0, 0]);
}

#[test]
fn test_opcode_execute_multiply() {
    let mut mem = vec![2, 3, 0, 3];
    let op = OpCode::Multiply;
    assert!(op.execute(0, &mut mem).is_ok());
    assert_eq!(mem, vec![2, 3, 0, 6]);
}