use crate::day2::op::OpCode;
use crate::day2::IntCodeMachine;
use crate::day2::*;
use std::convert::From;
use std::io::Write;
use std::num::ParseIntError;
use std::str::FromStr;

pub fn build_machine(mem: Vec<i64>) -> IntCodeMachine {
    let mut m = IntCodeMachine::boot(mem);
    m.reg_opcode(op::Input::code(), op::Input::new);
    m.reg_opcode(op::Output::code(), op::Output::new);
    m.reg_opcode(op::Jnz::code(), op::Jnz::new);
    m.reg_opcode(op::Jz::code(), op::Jz::new);
    m.reg_opcode(op::Eq::code(), op::Eq::new);
    m.reg_opcode(op::Lt::code(), op::Lt::new);
    m.reg_param_mode(1, immediate::load, immediate::store);
    m
}

pub fn run() -> Result<(), Error> {
    let instrs = read_comma_file("input/day5.txt")?;

    let m = build_machine(instrs);
    let _end_state = m.run()?;
    Ok(())
}

pub fn read<T>(prompt: &str) -> Result<T, Error>
where
    T: FromStr,
    ParseIntError: From<<T as FromStr>::Err>,
{
    print!("{} ", prompt);
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer)?;

    buffer
        .trim_end()
        .parse::<T>()
        .map_err(|e| Error::ParseIntError(e.into()))
}

pub mod op {
    use super::read;
    use crate::day2::op::OpCode;
    use crate::day2::param::{decompose_param, ParamReg};
    use crate::day2::{Error, LoadPtr, StorePtr};

    use std::sync::mpsc::{Receiver, Sender};

    pub use eq::*;
    pub use jnz::*;
    pub use jz::*;
    pub use lt::*;

    pub struct Input(StorePtr);
    impl OpCode for Input {
        fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
            let ps = decompose_param(param, Input::width() as usize);
            Ok(Box::new(Input(reg.get(ps[0])?.store)) as Box<dyn OpCode>)
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            _: &Option<Receiver<i64>>,
            _: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error> {
            let result = read::<i64>("INPUT: ")?;
            debug!("STDIN {}", result);
            self.0(ip + 1, mem, result, *rel_base)?;
            Ok(Input::width() as isize)
        }

        fn code() -> i64 {
            3
        }

        fn width() -> usize {
            2
        }
    }

    impl std::fmt::Debug for Input {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "INPUT")
        }
    }

    pub struct Output(LoadPtr);
    impl OpCode for Output {
        fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
            let ps = decompose_param(param, Output::width());
            Ok(Box::new(Output(reg.get(ps[0])?.load)) as Box<dyn OpCode>)
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            _: &Option<Receiver<i64>>,
            _: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error> {
            debug!("OUTPUT: {}", self.0(ip + 1, mem, *rel_base)?);
            Ok(Output::width() as isize)
        }

        fn code() -> i64 {
            4
        }

        fn width() -> usize {
            2
        }
    }

    impl std::fmt::Debug for Output {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "OUTPUT")
        }
    }

    pub mod jnz {
        use super::*;

        pub struct Jnz(pub LoadPtr, pub LoadPtr);
        impl OpCode for Jnz {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Jnz::width());
                Ok(Box::new(Jnz(reg.get(ps[0])?.load, reg.get(ps[1])?.load)) as Box<dyn OpCode>)
            }

            fn execute(
                &self,
                ip: isize,
                mem: &mut [i64],
                _: &Option<Receiver<i64>>,
                _: &mut Option<Sender<i64>>,
                rel_base: &mut isize,
            ) -> Result<isize, Error> {
                let value = self.0(ip + 1, mem, *rel_base)?;
                if value != 0 {
                    let dest = self.1(ip + 2, mem, *rel_base)?;
                    debug!("JNZ {} != 0 -> {}", value, dest);
                    Ok(dest as isize - ip)
                } else {
                    debug!("JNZ {} == 0", value);
                    Ok(Jnz::width() as isize)
                }
            }

            fn code() -> i64 {
                5
            }

            fn width() -> usize {
                3
            }
        }

        impl std::fmt::Debug for Jnz {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "JNZ")
            }
        }

        #[cfg(test)]
        mod jnz_test {
            use super::*;
            use crate::day5::immediate;
            use std::sync::mpsc::channel;

            #[test]
            fn new() {
                let mut reg = ParamReg::new();
                reg.register_mode(1, immediate::load, immediate::store);

                let op = Jnz::new(&reg, 11);
                assert!(op.is_ok());
            }

            #[test]
            fn no_jump() {
                let op = Jnz(immediate::load, immediate::load);
                let (tx, rx) = channel();

                // do not jump, ip = 3
                let mut mem = vec![1105, 0, 0];
                let r = op.execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(3));
            }

            #[test]
            fn jump_to_0() {
                let op = Jnz(immediate::load, immediate::load);
                let (tx, rx) = channel();
                // jump, ip = 0
                let mut mem = vec![1105, 1, 0];
                let r = op.execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(0));
            }

            #[test]
            fn jump_back() {
                let op = Jnz(immediate::load, immediate::load);
                let (tx, rx) = channel();
                // jump backwards, ip = 0
                let mut mem = vec![0, 0, 1105, 1, 0];
                let r = op.execute(2, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(-2));
            }
        }
    }

    pub mod jz {
        use super::*;
        use std::sync::mpsc::{Receiver, Sender};

        pub struct Jz(LoadPtr, LoadPtr);
        impl OpCode for Jz {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Jz::width());
                Ok(Box::new(Jz(reg.get(ps[0])?.load, reg.get(ps[1])?.load)) as Box<dyn OpCode>)
            }

            fn execute(
                &self,
                ip: isize,
                mem: &mut [i64],
                _: &Option<Receiver<i64>>,
                _: &mut Option<Sender<i64>>,
                rel_base: &mut isize,
            ) -> Result<isize, Error> {
                let value = self.0(ip + 1, mem, *rel_base)?;
                if value == 0 {
                    let dest = self.1(ip + 2, mem, *rel_base)?;
                    debug!("JZ {} == 0 -> {}", value, dest);
                    Ok(dest as isize - ip)
                } else {
                    debug!("JZ {} != 0", value);
                    Ok(Jz::width() as isize)
                }
            }

            fn code() -> i64 {
                6
            }

            fn width() -> usize {
                3
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;
            use crate::day5::immediate;
            use std::sync::mpsc::channel;

            #[test]
            fn jz() {
                let op = Jz(immediate::load, immediate::load);

                // jump, ip = 0
                let (tx, rx) = channel();
                let mut mem = vec![115, 0, 0];
                let r = op.execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(0));

                // do not jump, ip = 3
                let (tx, rx) = channel();
                let mut mem = vec![115, 1, 0];
                let r = op.execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(3));

                // jump backwards, ip = 0
                let (tx, rx) = channel();
                let mut mem = vec![0, 0, 115, 0, 0];
                let r = op.execute(2, &mut mem, &Some(rx), &mut Some(tx), &mut 0);
                assert_eq!(r, Ok(-2));
            }
        }

        impl std::fmt::Debug for Jz {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "JZ")
            }
        }
    }

    pub mod lt {
        use super::*;

        pub struct Lt(LoadPtr, LoadPtr, StorePtr);
        impl OpCode for Lt {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Lt::width());
                Ok(Box::new(Lt(
                    reg.get(ps[0])?.load,
                    reg.get(ps[1])?.load,
                    reg.get(ps[2])?.store,
                )) as Box<dyn OpCode>)
            }

            fn execute(
                &self,
                ip: isize,
                mem: &mut [i64],
                _: &Option<Receiver<i64>>,
                _: &mut Option<Sender<i64>>,
                rel_base: &mut isize,
            ) -> Result<isize, Error> {
                let l = self.0(ip + 1, mem, *rel_base)?;
                let r = self.1(ip + 2, mem, *rel_base)?;
                if l < r {
                    debug!("{} < {}", l, r);
                    self.2(ip + 3, mem, 1, *rel_base)?;
                } else {
                    debug!("{} > {}", l, r);
                    self.2(ip + 3, mem, 0, *rel_base)?;
                }
                Ok(Lt::width() as isize)
            }

            fn code() -> i64 {
                7
            }

            fn width() -> usize {
                4
            }
        }

        impl std::fmt::Debug for Lt {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "LT")
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;
            use crate::day2::indirect::*;
            use std::sync::mpsc::channel;

            #[test]
            fn test_lt() {
                // true, write 1 to @3
                let mut mem = vec![7, 4, 5, 3, 1, 2];
                let (tx, rx) = channel();
                let lt = Lt(load, load, store);
                assert!(lt
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![7, 4, 5, 1, 1, 2]);

                // false, write 0 to @3
                let mut mem = vec![7, 5, 4, 3, 1, 2];
                let (tx, rx) = channel();
                let lt = Lt(load, load, store);
                assert!(lt
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![7, 5, 4, 0, 1, 2]);
            }
        }
    }

    pub mod eq {
        use super::*;
        use std::sync::mpsc::{Receiver, Sender};

        pub struct Eq(LoadPtr, LoadPtr, StorePtr);
        impl OpCode for Eq {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Eq::width());
                Ok(Box::new(Eq(
                    reg.get(ps[0])?.load,
                    reg.get(ps[1])?.load,
                    reg.get(ps[2])?.store,
                )) as Box<dyn OpCode>)
            }

            fn execute(
                &self,
                ip: isize,
                mem: &mut [i64],
                _: &Option<Receiver<i64>>,
                _: &mut Option<Sender<i64>>,
                rel_base: &mut isize,
            ) -> Result<isize, Error> {
                let l = self.0(ip + 1, mem, *rel_base)?;
                let r = self.1(ip + 2, mem, *rel_base)?;
                if l == r {
                    debug!("{} == {}", l, r);
                    self.2(ip + 3, mem, 1, *rel_base)?;
                } else {
                    debug!("{} != {}", l, r);
                    self.2(ip + 3, mem, 0, *rel_base)?;
                }
                Ok(Eq::width() as isize)
            }

            fn code() -> i64 {
                8
            }

            fn width() -> usize {
                4
            }
        }

        impl std::fmt::Debug for Eq {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "EQ")
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;
            use crate::day2::indirect::*;
            use std::sync::mpsc::channel;

            #[test]
            fn test_eq() {
                let mut mem = vec![118, 1, 2, 3];
                let (tx, rx) = channel();
                let lt = Eq(load, load, store);
                assert!(lt
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![118, 1, 2, 0]);

                let mut mem = vec![118, 1, 1, 3];
                let (tx, rx) = channel();
                let lt = Eq(load, load, store);
                assert!(lt
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![118, 1, 1, 1]);
            }
        }
    }
}

pub mod immediate {
    use super::Error;

    pub fn load(ptr: isize, mem: &[i64], _rel_base: isize) -> Result<i64, Error> {
        assert!(ptr >= 0);
        let value = *mem.get(ptr as usize).ok_or(Error::MemoryError(ptr))?;
        debug!("IMM LD ${}", value);
        Ok(value)
    }

    pub fn store(
        _ptr: isize,
        _mem: &mut [i64],
        _value: i64,
        _rel_base: isize,
    ) -> Result<(), Error> {
        unreachable!();
        // *mem.get_mut(ptr).ok_or(Error::MemoryError)? = value;
        // Ok(())
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn imm_get() {
            let mem = vec![12, 0];
            assert_eq!(load(1, &mem, 0), Ok(0));
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use super::op;
    use super::*;
    use crate::day2::op::add::Add;
    use crate::day2::op::OpCode;
    use crate::day2::param::ParamReg;
    use crate::day2::{indirect, Error};

    use mopa::mopafy;
    use std::sync::mpsc::{Receiver, Sender};

    mopafy!(OpCode);

    // the output is stored in the inner value
    pub struct MockOutput(pub RefCell<Option<i64>>);
    impl OpCode for MockOutput {
        fn new(_reg: &ParamReg, _param: i64) -> Result<Box<dyn OpCode>, Error> {
            Ok(Box::new(MockOutput(RefCell::new(None))))
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            _: &Option<Receiver<i64>>,
            _: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error> {
            *self.0.borrow_mut() = Some(indirect::load(ip + 1, mem, *rel_base)?);
            Ok(MockOutput::width() as isize)
        }

        fn width() -> usize {
            2
        }

        fn code() -> i64 {
            4
        }
    }

    impl std::fmt::Debug for MockOutput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FAKE_OUT")
        }
    }

    // the param arg to the input is used as the value to be stored
    pub struct MockInput(i64);
    impl OpCode for MockInput {
        fn new(_reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
            Ok(Box::new(MockInput(param)))
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            _: &Option<Receiver<i64>>,
            _: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error> {
            indirect::store(ip + 1, mem, self.0, *rel_base)?;
            Ok(MockInput::width() as isize)
        }

        fn width() -> usize {
            2
        }

        fn code() -> i64 {
            3
        }
    }

    impl std::fmt::Debug for MockInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FAKE_IN")
        }
    }

    #[test]
    fn imm_mode_decode() {
        let mut m = IntCodeMachine::boot(Vec::new());
        m.reg_param_mode(1, immediate::load, immediate::store);
        let op = m.decode(1101);
        assert!(op.is_ok(), format!("op is {:?}", op));
        let op = op.unwrap();

        match op.downcast_ref::<Add>() {
            Some(as_add) => {
                assert_eq!(as_add.0 as usize, immediate::load as usize);
                assert_eq!(as_add.1 as usize, immediate::load as usize);
                assert_eq!(as_add.2 as usize, indirect::store as usize);
            }
            None => assert!(false, format!("Decoded {} = {:?}", print_type_of(&op), op)),
        }
    }

    #[test]
    fn test_run_eq_ind() {
        let mem = vec![803, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Eq::code(), op::Eq::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![803, 9, 8, 9, 10, 9, 4, 9, 99, 1, 8]));
    }

    #[test]
    fn test_run_lt_ind() {
        let mem = vec![803, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Lt::code(), op::Lt::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![803, 9, 7, 9, 10, 9, 4, 9, 99, 0, 8]));
    }

    #[test]
    fn test_run_eq_imm() {
        let mem = vec![803, 3, 1108, -1, 8, 3, 4, 3, 99];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Eq::code(), op::Eq::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![803, 3, 1108, 1, 8, 3, 4, 3, 99]));

        let mem = vec![903, 3, 1108, -1, 8, 3, 4, 3, 99];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Eq::code(), op::Eq::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![903, 3, 1108, 0, 8, 3, 4, 3, 99]));
    }

    #[test]
    fn test_run_lt_imm() {
        let mem = vec![803, 3, 1107, -1, 8, 3, 4, 3, 99];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Lt::code(), op::Lt::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![803, 3, 1107, 0, 8, 3, 4, 3, 99]));

        let mem = vec![303, 3, 1107, -1, 8, 3, 4, 3, 99];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Lt::code(), op::Lt::new);
        let end_state = m.run();
        assert_eq!(end_state, Ok(vec![303, 3, 1107, 1, 8, 3, 4, 3, 99]));
    }

    #[test]
    fn test_jmp_int() {
        let mem = vec![503, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Jz::code(), op::Jz::new);
        let end_state = m.run();
        assert_eq!(
            end_state,
            Ok(vec![
                503, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, 5, 1, 1, 9
            ])
        );

        let mem = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Jz::code(), op::Jz::new);
        let end_state = m.run();
        assert_eq!(
            end_state,
            Ok(vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, 0, 0, 1, 9])
        );
    }

    #[test]
    fn test_jmp_imm() {
        let mem = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Jnz::code(), op::Jnz::new);
        let end_state = m.run();
        assert_eq!(
            end_state,
            Ok(vec![3, 3, 1105, 0, 9, 1101, 0, 0, 12, 4, 12, 99, 0])
        );

        let mem = vec![103, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_param_mode(1, immediate::load, immediate::store);
        m.reg_opcode(MockInput::code(), MockInput::new);
        m.reg_opcode(MockOutput::code(), MockOutput::new);
        m.reg_opcode(op::Jnz::code(), op::Jnz::new);
        let end_state = m.run();
        assert_eq!(
            end_state,
            Ok(vec![103, 3, 1105, 1, 9, 1101, 0, 0, 12, 4, 12, 99, 1])
        );
    }

    fn print_type_of<T>(_: &T) -> String {
        format!("{}", std::any::type_name::<T>())
    }
}
