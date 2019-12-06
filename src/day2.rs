use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{BufRead, BufReader};

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

impl TryFrom<u32> for Box<dyn OpCode> {
    type Error = Error;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
        Ok(match i {
            1 => Box::new(ops::Add),
            2 => Box::new(ops::Multiply),
            99 => Box::new(ops::Terminate),
            _ => Err(Error::InvalidInstruction)?,
        })
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
            execute(&mut instrs)?;
            if instrs[0] == 19690720 {
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

/// Execute the given memory
fn execute(memory: &mut [u32]) -> Result<(), Error> {
    let mut ip = 0;
    loop {
        let op: Box<dyn OpCode> = memory[ip].try_into()?;
        match op.execute(ip, memory) {
            Ok(d) => ip += d,
            Err(Error::Terminated) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
