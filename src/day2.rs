use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::*;

use op::add::Add;
use op::mul::Mul;
use op::term::Term;
use op::OpCode;
use param::ParamReg;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    BadOpcode(i64),
    MemoryError(isize),
    BadParamMode,
    Terminated,
    /// I/O error
    IoError(std::io::ErrorKind),
    /// Could not parse number in input
    ParseIntError(std::num::ParseIntError),
    /// Program needs input
    NeedsInput,
    /// Could not read from input pipe
    InputFailed,
    /// Program has output
    HasOutput(i64),
    /// Could not write to output pipe
    OutputFailed,
    UserInputFailed,
    NotRunning,
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_e: std::sync::mpsc::SendError<T>) -> Self {
        Error::UserInputFailed
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e.kind())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(e)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(_: std::sync::mpsc::RecvError) -> Self {
        Error::InputFailed
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn read_comma_file(filename: &str) -> Result<Vec<i64>, Error> {
    BufReader::new(File::open(filename)?)
        .split(b',')
        .map(|s| {
            std::str::from_utf8(&s.unwrap())
                .unwrap()
                .trim_end()
                .to_string()
        })
        .map(|s| i64::from_str_radix(&s, 10).map_err(|e| e.into()))
        .collect()
}

/// Run day 2
pub fn run() -> Result<String, Error> {
    let instrs = read_comma_file("input/day2.txt")?;

    let mut noun = None;
    let mut verb = None;
    for n in 0..=99 {
        for v in 0..=99 {
            let mut instrs = instrs.clone();
            instrs[1] = n;
            instrs[2] = v;
            let machine = IntCodeMachine::boot(instrs);
            let end_state = machine.run()?;
            if end_state[0] == 19690720 {
                noun = Some(n);
                verb = Some(v);
                break;
            }
        }
    }

    if noun.is_some() && verb.is_some() {
        Ok(format!("{}", 100 * noun.unwrap() + verb.unwrap()))
    } else {
        Ok("not found".to_string())
    }
}

pub struct IntCodeMachine {
    ip: isize,
    mem: Vec<i64>,
    op_map: HashMap<i64, fn(&ParamReg, i64) -> Result<Box<dyn OpCode>, Error>>,
    p_reg: ParamReg,
    input: Option<Receiver<i64>>,
    output: Option<Sender<i64>>,
    user_input: Option<Sender<i64>>,
    rel_base: isize,
}

impl std::fmt::Debug for IntCodeMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ICM: ip={}, mem=[{}], rel_base={}, known_ops=[{}], known_pmodes=[{}], i?={}, o?={}",
            self.ip,
            self.mem
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .as_slice()
                .join(", "),
            self.rel_base,
            self.op_map
                .keys()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .as_slice()
                .join(", "),
            self.p_reg
                .mode_map
                .keys()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .as_slice()
                .join(", "),
            if self.input.is_some() { "y" } else { "n" },
            if self.output.is_some() { "y" } else { "n" },
        )
    }
}

impl IntCodeMachine {
    pub fn boot(mem: Vec<i64>) -> Self {
        let mut m = IntCodeMachine {
            ip: 0,
            mem,
            op_map: HashMap::new(),
            p_reg: ParamReg::new(),
            input: None,
            output: None,
            user_input: None,
            rel_base: 0,
        };
        m.reg_opcode(Add::code(), Add::new);
        m.reg_opcode(Mul::code(), Mul::new);
        m.reg_opcode(Term::code(), Term::new);
        m.reg_param_mode(0, indirect::load, indirect::store);
        m
    }

    fn step(&mut self) -> Result<(), Error> {
        let op = self.decode(
            *self
                .mem
                .get(self.ip as usize)
                .ok_or(Error::MemoryError(self.ip))?,
        )?;
        let orig_ip_val = *self
            .mem
            .get(self.ip as usize)
            .ok_or(Error::MemoryError(self.ip))?;
        let diff = op.execute(
            self.ip,
            &mut self.mem,
            &self.input,
            &mut self.output,
            &mut self.rel_base,
        )?;

        println!("{:?}\n", &op);
        /*
        println!(
            "{:?}\t{}\n",
            &op,
            self.mem[self.ip as usize..(self.ip + 4) as usize]
                .iter()
                .map(|v| format!("{}", *v))
                .collect::<Vec<String>>()
                .as_slice()
                .join(" ")
        );
        */

        if orig_ip_val
            != *self
                .mem
                .get(self.ip as usize)
                .ok_or(Error::MemoryError(self.ip))?
        {
            // the value under the IP was written - jump to that address
            self.ip = *self
                .mem
                .get(self.ip as usize)
                .ok_or(Error::MemoryError(self.ip))? as isize;
        } else {
            self.ip += diff;
        }
        // dbg!(&op, orig_ip_val, diff, self.ip);

        if self.ip >= 0 {
            Ok(())
        } else {
            // dbg!(self.ip);
            Err(Error::MemoryError(self.ip))
        }
    }

    pub fn run(mut self) -> Result<Vec<i64>, Error> {
        loop {
            match self.step() {
                Ok(_) => {
                    //dbg!(&self);
                    continue;
                }
                Err(Error::Terminated) => {
                    //dbg!(&self);
                    println!("Terminated gracefully.");
                    break Ok(self.mem);
                }
                Err(e) => {
                    dbg!(&self);
                    break dbg!(Err(e));
                }
            }
        }
    }

    pub fn decode(&mut self, opcode: i64) -> Result<Box<dyn OpCode>, Error> {
        let (op, param) = (opcode % 100, opcode / 100);
        self.op_map.get(&op).ok_or(Error::BadOpcode(op))?(&self.p_reg, param)
    }

    pub fn reg_opcode(
        &mut self,
        opcode: i64,
        ctor: fn(&ParamReg, i64) -> Result<Box<dyn OpCode>, Error>,
    ) {
        self.op_map.insert(opcode, ctor);
    }

    pub fn reg_param_mode(&mut self, id: i64, load: LoadPtr, store: StorePtr) {
        self.p_reg.register_mode(id, load, store);
    }

    pub fn wire_input(&mut self) -> Sender<i64> {
        let (tx, rx) = channel();
        self.user_input = Some(tx.clone());
        self.input = Some(rx);
        tx
    }

    pub fn wire_output(&mut self, tx: Sender<i64>) {
        self.output = Some(tx);
    }

    pub fn get_input_handle(&self) -> Option<Sender<i64>> {
        if let Some(ch) = &self.user_input {
            Some(ch.clone())
        } else {
            None
        }
    }
}

pub type LoadPtr = fn(isize, &[i64], isize) -> Result<i64, Error>;
pub type StorePtr = fn(isize, &mut [i64], i64, isize) -> Result<(), Error>;

pub mod op {
    use super::param::{decompose_param, ParamReg};
    use super::{Error, LoadPtr, StorePtr};
    use mopa::Any;

    use std::sync::mpsc::{Receiver, Sender};

    pub trait OpCode: std::fmt::Debug + Any {
        fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error>
        where
            Self: Sized;

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            inp: &Option<Receiver<i64>>,
            out: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error>;

        fn code() -> i64
        where
            Self: Sized;

        fn width() -> usize
        where
            Self: Sized;
    }

    pub mod mul {
        use super::*;

        pub struct Mul(LoadPtr, LoadPtr, StorePtr);
        impl OpCode for Mul {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Mul::width() as usize);
                Ok(Box::new(Mul(
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
                let result = l * r;
                println!("{} * {} = {}", l, r, result);
                self.2(ip + 3, mem, result, *rel_base)?;
                Ok(Mul::width() as isize)
            }

            fn code() -> i64 {
                2
            }

            fn width() -> usize {
                4
            }
        }

        impl std::fmt::Debug for Mul {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MUL")
            }
        }

        #[cfg(test)]
        mod mul_test {
            use super::Mul;
            use crate::day2::indirect::*;
            use crate::day2::op::OpCode;
            use std::sync::mpsc::channel;

            #[test]
            fn mul() {
                let (tx, rx) = channel();
                let mut mem = vec![2, 0, 0, 4, 0];
                let mul = Mul(load, load, store);
                assert!(mul
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![2, 0, 0, 4, 4]);
            }
        }
    }

    pub mod add {
        use super::*;

        pub struct Add(pub LoadPtr, pub LoadPtr, pub StorePtr);

        impl OpCode for Add {
            fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
                let ps = decompose_param(param, Add::width() as usize);
                Ok(Box::new(Add(
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
                let result = l + r;
                println!("{} + {} = {}", l, r, result);
                self.2(ip + 3, mem, result, *rel_base)?;
                Ok(Add::width() as isize)
            }

            fn code() -> i64 {
                1
            }

            fn width() -> usize {
                4
            }
        }

        impl std::fmt::Debug for Add {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "ADD")
            }
        }

        #[cfg(test)]
        mod add_test {
            use super::Add;
            use crate::day2::indirect::*;
            use crate::day2::op::OpCode;
            use std::sync::mpsc::channel;

            #[test]
            fn test_add() {
                let (tx, rx) = channel();
                let mut mem = vec![1, 0, 0, 4, 0];
                let add = Add(load, load, store);
                assert!(add
                    .execute(0, &mut mem, &Some(rx), &mut Some(tx), &mut 0)
                    .is_ok());
                assert_eq!(mem, vec![1, 0, 0, 4, 2]);
            }
        }
    }

    pub mod term {
        use super::*;

        pub struct Term;

        impl OpCode for Term {
            fn new(_reg: &ParamReg, _param: i64) -> Result<Box<dyn OpCode>, Error> {
                Ok(Box::new(Term) as Box<dyn OpCode>)
            }

            fn execute(
                &self,
                _ip: isize,
                _mem: &mut [i64],
                _: &Option<Receiver<i64>>,
                _: &mut Option<Sender<i64>>,
                _: &mut isize,
            ) -> Result<isize, Error> {
                println!("TERM");
                Err(Error::Terminated)
            }

            fn code() -> i64 {
                99
            }

            fn width() -> usize {
                unreachable!();
            }
        }

        impl std::fmt::Debug for Term {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "TERM")
            }
        }
    }
}

pub mod indirect {
    use super::Error;

    pub fn load(ptr: isize, mem: &[i64], _: isize) -> Result<i64, Error> {
        assert!(ptr >= 0);
        let iptr: isize = *mem.get(ptr as usize).ok_or(Error::MemoryError(ptr))? as isize;
        let value = *mem.get(iptr as usize).ok_or(Error::MemoryError(ptr))?;
        println!("IND LD @{} {}", iptr, value);
        Ok(value)
    }

    pub fn store(ptr: isize, mem: &mut [i64], value: i64, _: isize) -> Result<(), Error> {
        assert!(ptr >= 0);
        let iptr = *mem.get(ptr as usize).ok_or(Error::MemoryError(ptr))? as isize;
        *mem.get_mut(iptr as usize).ok_or(Error::MemoryError(ptr))? = value;
        println!("IND STO @{} {}", iptr, value);
        Ok(())
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn indir_get() {
            let mem = vec![12, 0];
            assert_eq!(load(1, &mem, 0), Ok(12));
        }

        #[test]
        fn indir_store() {
            let mut mem = vec![12, 0];
            assert!(store(1, &mut mem, 42, 0).is_ok());
            assert_eq!(mem, vec![42, 0]);
        }
    }
}

pub struct LSPair {
    pub load: LoadPtr,
    pub store: StorePtr,
}

pub mod param {
    use super::Error;
    use super::{LSPair, LoadPtr, StorePtr};
    use std::collections::HashMap;

    pub struct ParamReg {
        pub mode_map: HashMap<i64, LSPair>,
    }

    impl ParamReg {
        pub fn new() -> Self {
            ParamReg {
                mode_map: HashMap::new(),
            }
        }

        pub fn get(&self, mode: i64) -> Result<&LSPair, Error> {
            self.mode_map.get(&mode).ok_or(Error::BadParamMode)
        }

        pub fn register_mode(&mut self, id: i64, load: LoadPtr, store: StorePtr) {
            self.mode_map.insert(id, LSPair { load, store });
        }
    }

    pub fn decompose_param(mut code: i64, width: usize) -> Vec<i64> {
        let mut v = Vec::new();
        while code > 0 {
            v.push(code % 10);
            code /= 10;
        }
        let v_len = v.len();
        v.into_iter()
            .chain(vec![0; width - v_len].into_iter())
            .collect()
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn testdecompose_param() {
            assert_eq!(decompose_param(0, 4), vec![0, 0, 0, 0]);
            assert_eq!(decompose_param(1100, 4), vec![0, 0, 1, 1]);
        }
    }
}

#[cfg(test)]
mod run_test {
    use super::*;

    #[test]
    fn test_execute() {
        // add @0 + @0 -> @5 => 1 + 1 = 2
        let program = vec![1, 0, 0, 5, 99, 5];
        let r = IntCodeMachine::boot(program).run();
        assert!(r.is_ok(), format!("r is {:?}", r));
        assert_eq!(r.unwrap(), vec![1, 0, 0, 5, 99, 2]);

        let program = vec![2, 3, 0, 3, 99];
        let r = IntCodeMachine::boot(program).run();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), vec![2, 3, 0, 6, 99]);

        let program = vec![2, 4, 4, 5, 99, 0];
        let r = IntCodeMachine::boot(program).run();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), vec![2, 4, 4, 5, 99, 9801]);

        let program = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        let r = IntCodeMachine::boot(program).run();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), vec![30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }
}
