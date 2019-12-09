use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Day 2 related errors
#[derive(PartialEq, Debug)]
pub enum Error {
    /// Got an invalid opcode value
    InvalidInstruction,
    /// Got a bad parameter mode value
    InvalidParamMode(u8),
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
    fn execute(
        &self,
        ip: usize,
        argmodes: Vec<ParamMode>,
        memory: &mut [i32],
    ) -> Result<usize, Error>;
}

pub fn decode(op: i32) -> Result<(i32, Vec<ParamMode>), Error> {
    let op_part = op % 100; // get rightmost two digits
    let mut op = op / 100;
    let mut modes = Vec::new();
    while op > 0 {
        modes.push(match (op % 10) as u8 {
            0u8 => ParamMode::Indirect,
            1u8 => ParamMode::Immediate,
            n @ _ => return Err(Error::InvalidParamMode(n)),
        });
        op /= 10;
    }
    Ok((op_part, modes))
}

#[derive(Clone)]
pub enum ParamMode {
    Immediate,
    Indirect,
}

impl ParamMode {
    pub fn access<'a>(&self, ptr: usize, mem: &'a mut [i32]) -> Result<&'a mut i32, Error> {
        match self {
            ParamMode::Immediate => mem.get_mut(ptr).ok_or(Error::MemoryError),
            ParamMode::Indirect => mem
                .get_mut(*mem.get(ptr).ok_or(Error::MemoryError)? as usize)
                .ok_or(Error::MemoryError),
        }
    }
}

pub mod ops {
    use super::ParamMode;
    use super::{Error, OpCode};
    use super::{IndirectFetch, IndirectStore};

    fn munge_params(width: usize, modes: Vec<ParamMode>) -> Vec<ParamMode> {
        modes
            .into_iter()
            .chain(vec![ParamMode::Indirect; width - modes.len()].into_iter())
            .rev()
            .collect()
    }

    pub struct Add;
    impl OpCode for Add {
        fn execute(
            &self,
            ip: usize,
            argmodes: Vec<ParamMode>,
            memory: &mut [i32],
        ) -> Result<usize, Error> {
            let left = *argmodes[0].access(ip + 1, memory)?;
            let left = memory.get_at(ip + 1)?;
            let right = memory.get_at(ip + 2)?;
            let result = left + right;
            memory.store_at(ip + 3, result)?;
            Ok(4)
        }
    }

    pub struct Multiply;
    impl OpCode for Multiply {
        fn execute(
            &self,
            ip: usize,
            argmodes: Vec<ParamMode>,
            memory: &mut [i32],
        ) -> Result<usize, Error> {
            let left = memory.get_at(ip + 1)?;
            let right = memory.get_at(ip + 2)?;
            memory.store_at(ip + 3, left * right)?;
            Ok(4)
        }
    }

    pub struct Terminate;
    impl OpCode for Terminate {
        fn execute(
            &self,
            _ip: usize,
            _argmodes: Vec<ParamMode>,
            _memory: &mut [i32],
        ) -> Result<usize, Error> {
            Err(Error::Terminated)
        }
    }
}

pub trait Load {
    fn load(self, ptr: usize) -> Result<i32, Error>;
}

pub struct ImmediateLoad;
impl Load for ImmediateLoad {
    fn load(ptr: usize, mem: &[i32]) -> Result<&i32, Error> {}
}

pub trait IndirectFetch {
    fn get_at(self, ptr: usize) -> Result<i32, Error>;
}

impl IndirectFetch for &mut [i32] {
    fn get_at(self, ptr: usize) -> Result<i32, Error> {
        Ok(*self
            .get(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)?)
    }
}

pub trait IndirectStore {
    fn store_at(self, ptr: usize, value: i32) -> Result<(), Error>;
}

impl IndirectStore for &mut [i32] {
    fn store_at(self, ptr: usize, value: i32) -> Result<(), Error> {
        *self
            .get_mut(*self.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)? = value;

        Ok(())
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
            ip: 0usize,
        }
    }

    pub fn register_opcode(&mut self, op_byte: i32, opcode: Box<dyn OpCode>) {
        self.opcodes.insert(op_byte, opcode);
    }

    pub fn step(&mut self) -> Result<&[i32], Error> {
        let (opcode, params) = decode(*self.mem.get(self.ip).ok_or(Error::MemoryError)?)?;
        self.ip += self
            .opcodes
            .get(&opcode)
            .ok_or(Error::InvalidInstruction)?
            .execute(self.ip, params, &mut self.mem)?;
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
        .map(|s| i32::from_str_radix(&s, 10).map_err(|e| e.into()))
        .collect::<Result<Vec<i32>, Error>>()?;

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
