use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

/// Day 2 related errors
#[derive(PartialEq, Debug)]
pub enum Error {
    /// Got an invalid opcode value
    InvalidInstruction,
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
    fn execute(&self, ip: usize, memory: &mut [u32]) -> Result<usize, Error>;
}

pub mod ops {
    use super::{Error,OpCode};
    use super::{IndirectFetch, IndirectStore};

    pub struct Add;
    impl OpCode for Add {
        fn execute(&self, ip: usize, memory: &mut [u32]) -> Result<usize, Error> {
            let left = memory.get_at(ip+1)?;
            let right = memory.get_at(ip + 2)?;
            let result = left + right;
            memory.store_at(ip + 3, result)?;
            Ok(4)
        }
    }

    pub struct Multiply;
    impl OpCode for Multiply {
        fn execute(&self, ip: usize, memory: &mut [u32]) -> Result<usize, Error> {
            let left = memory.get_at(ip+1)?;
            let right = memory.get_at(ip+2)?;
            memory.store_at(ip + 3, left * right)?;
            Ok(4)
        }
    }

    pub struct Terminate;
    impl OpCode for Terminate {
        fn execute(&self, _ip: usize, _memory: &mut [u32]) -> Result<usize, Error> {
            Err(Error::Terminated)
        }
    }

}

pub trait IndirectFetch {
    fn get_at(self, ptr: usize) -> Result<u32, Error>;
}

impl IndirectFetch for &mut [u32] {
    fn get_at(self, ptr: usize) -> Result<u32, Error> {
        Ok(*self
            .get(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)?)

    }
}

pub trait IndirectStore {
    fn store_at(self, ptr: usize, value: u32) -> Result<(), Error>;
}

impl IndirectStore for &mut [u32] {
    fn store_at(self, ptr: usize, value: u32) -> Result<(), Error> {
        *self.get_mut(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)? = value;

        Ok(())
    }
}

pub struct IntCodeMachine {
    mem: Vec<u32>,
    opcodes: HashMap<u32, Box<dyn OpCode>>,
    ip: usize,
}

impl IntCodeMachine {
    pub fn boot(mem: Vec<u32>) -> Self {
        let mut opcodes: HashMap<u32, Box<dyn OpCode>> = HashMap::new();
        opcodes.insert(1, Box::new(ops::Add));
        opcodes.insert(2, Box::new(ops::Multiply));
        opcodes.insert(99, Box::new(ops::Terminate));

        IntCodeMachine {
            mem,
            opcodes,
            ip: 0usize
        }
    }

    pub fn register_opcode(&mut self, op_byte: u32, opcode: Box<dyn OpCode>) {
        self.opcodes.insert(op_byte, opcode);
    }

    pub fn step(&mut self) -> Result<&[u32], Error> {
        self.ip += self
            .opcodes
            .get(
                self.mem.get(self.ip).ok_or(Error::MemoryError)?
                )
            .ok_or(Error::InvalidInstruction)?
            .execute(self.ip, &mut self.mem)?;
        Ok(&self.mem)
    }

    pub fn run(&mut self) -> Result<&[u32], Error> {
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

/// Run day 2
pub fn run() -> Result<String, Error> {
    let instrs = BufReader::new(File::open("input/day2.txt")?)
        .split(b',')
        .map(|s| {
            std::str::from_utf8(&s.unwrap())
                .unwrap()
                .trim_end()
                .to_string()
        })
        .map(|s| u32::from_str_radix(&s, 10).map_err(|e| e.into()))
        .collect::<Result<Vec<u32>, Error>>()?;

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
