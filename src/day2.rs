use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

/// Day 2 related errors
#[derive(PartialEq, Debug)]
pub enum Error {
    /// Got an invalid opcode value
    InvalidInstruction,
    /// Got an invalid parameter specifier
    InvalidParameter,
    /// Tried to access an out-of-bounds memory location
    MemoryError,
    /// Encountered termination opcode - not an error
    Terminated,
    /// I/O error
    IoError(std::io::ErrorKind),
    /// Could not parse number in input
    ParseIntError(std::num::ParseIntError),
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait OpCode {
    fn execute(&self, param_code: i32, ip: usize, memory: &mut [i32]) -> Result<usize, Error>;
}

pub mod ops {
    use super::{Error,OpCode};
    use super::mem::{Fetch, Store, ParamMode};

    pub struct Add;
    impl OpCode for Add {
        fn execute(&self, param_code: i32, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
            let params = decode(param_code, 4)?;
            let left = memory.fetch(ip+1, &params[0])?;
            let right = memory.fetch(ip + 2, &params[1])?;
            let result = left + right;
            assert!(params[2] != ParamMode::Immediate);
            memory.store(ip + 3, &params[2], result)?;
            Ok(4)
        }
    }

    pub struct Multiply;
    impl OpCode for Multiply {
        fn execute(&self, param_code: i32, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
            let params = decode(param_code, 4)?;
            let left = memory.fetch(ip+1, &params[0])?;
            let right = memory.fetch(ip+2, &params[1])?;
            assert!(params[2] != ParamMode::Immediate);
            memory.store(ip + 3, &params[2], left * right)?;
            Ok(4)
        }
    }

    pub struct Terminate;
    impl OpCode for Terminate {
        fn execute(&self, _param_code: i32, _ip: usize, _memory: &mut [i32]) -> Result<usize, Error> {
            Err(Error::Terminated)
        }
    }

    pub fn decode(mut param_code: i32, width: usize) -> Result<Vec<ParamMode>, Error> {
        let mut params = Vec::new();
        while param_code > 0 {
            params.push(match param_code % 10 {
                0 => ParamMode::Indirect,
                1 => ParamMode::Immediate,
                _ => Err(Error::InvalidParameter)?,
            });
            param_code /= 10;
        }
        let params_len = params.len();
        let result = params.into_iter().chain(vec![ParamMode::Indirect; width - params_len].into_iter()).collect();
        Ok(result)
    }

}

pub mod mem {
    use super::Error;

    #[derive(Clone, Debug, PartialEq)]
    pub enum ParamMode {
        Immediate,
        Indirect,
    }

    pub trait Fetch: ImmediateFetch + IndirectFetch {
        fn fetch(self, ptr: usize, mode: &ParamMode) -> Result<i32, Error>;
    }

    impl Fetch for &mut [i32] {
        fn fetch(self, ptr: usize, mode: &ParamMode) -> Result<i32, Error> {
            match mode {
                ParamMode::Immediate => self.imm_fetch(ptr),
                ParamMode::Indirect => self.fetch_at(ptr),
            }
        }
    }

    pub trait ImmediateFetch {
        fn imm_fetch(self, ptr: usize) -> Result<i32, Error>;
    }

    impl ImmediateFetch for &mut [i32] {
        fn imm_fetch(self, ptr: usize) -> Result<i32, Error> {
            Ok(*self.get(ptr).ok_or(Error::MemoryError)?)
        }
    }


    pub trait IndirectFetch {
        fn fetch_at(self, ptr: usize) -> Result<i32, Error>;
    }

    impl IndirectFetch for &mut [i32] {
        fn fetch_at(self, ptr: usize) -> Result<i32, Error> {
            Ok(*self
                .get(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
                .ok_or(Error::MemoryError)?)

        }
    }

    pub trait Store: IndirectStore + ImmediateStore {
        fn store(self, ptr: usize, mode: &ParamMode, value: i32) -> Result<(), Error>;
    }

    impl Store for &mut [i32] {
        fn store(self, ptr: usize, mode: &ParamMode, value: i32) -> Result<(), Error> {
            match mode {
                ParamMode::Immediate => self.imm_store(ptr, value),
                ParamMode::Indirect => self.store_at(ptr, value),
            }
        }
    }

    pub trait ImmediateStore {
        fn imm_store(self, ptr: usize, value: i32) -> Result<(), Error>;
    }

    impl ImmediateStore for &mut [i32] {
        fn imm_store(self, ptr: usize, value: i32) -> Result<(), Error> {
            *self.get_mut(ptr).ok_or(Error::MemoryError)? = value;
            Ok(())
        }
    }

    pub trait IndirectStore {
        fn store_at(self, ptr: usize, value: i32) -> Result<(), Error>;
    }

    impl IndirectStore for &mut [i32] {
        fn store_at(self, ptr: usize, value: i32) -> Result<(), Error> {
            *self.get_mut(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
                .ok_or(Error::MemoryError)? = value;

            Ok(())
        }
    }

}

pub struct IntCodeMachine {
    mem: Vec<i32>,
    opcodes: HashMap<i32, Box<dyn OpCode>>,
    ip: usize,
}

impl IntCodeMachine {
    pub fn boot(mem: Vec<i32>) -> Self {
        let mut opcodes: HashMap<i32, Box<dyn OpCode>> = HashMap::new();
        opcodes.insert(1, Box::new(ops::Add));
        opcodes.insert(2, Box::new(ops::Multiply));
        opcodes.insert(99, Box::new(ops::Terminate));

        IntCodeMachine {
            mem,
            opcodes,
            ip: 0usize
        }
    }

    pub fn register_opcode(&mut self, op_byte: i32, opcode: Box<dyn OpCode>) {
        self.opcodes.insert(op_byte, opcode);
    }

    pub fn step(&mut self) -> Result<&[i32], Error> {
        let (opcode, param_code) = (self.mem.get(self.ip).ok_or(Error::MemoryError)? % 100, self.mem.get(self.ip).ok_or(Error::MemoryError)? / 100);
        let op = self.opcodes.get(&opcode).ok_or(Error::MemoryError)?;

        self.ip += op.execute(param_code as i32, self.ip, &mut self.mem)?;

        Ok(&self.mem)
    }

    pub fn run(&mut self) -> Result<&[i32], Error> {
        loop {
            match self.step() {
                Ok(_) => continue,
                Err(Error::Terminated) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(&self.mem)
    }

}

pub fn read_comma_input(filename: &str) -> Result<Vec<i32>, Error> {
    BufReader::new(File::open(filename)?)
        .split(b',')
        .map(|s| {
            std::str::from_utf8(&s.unwrap())
                .unwrap()
                .trim_end()
                .to_string()
        })
        .map(|s| i32::from_str_radix(&s, 10).map_err(|e| e.into()))
        .collect::<Result<Vec<i32>, Error>>()
}

pub fn run() -> Result<String, Error> {
    let instrs = read_comma_input("input/day2.txt")?;

    let mut noun = None;
    let mut verb = None;
    for n in 0..=99 {
        for v in 0..=99 {
            let mut instrs = instrs.clone();
            instrs[1] = n;
            instrs[2] = v;
            let mut machine = IntCodeMachine::boot(instrs);
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

#[cfg(test)]
mod tests;
