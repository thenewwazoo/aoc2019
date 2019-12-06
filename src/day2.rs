use std::convert::TryFrom;
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

/// IntCode opcodes
pub enum OpCode {
    /// Add: args (&lhs, &rhs, &dest)
    Add,
    /// Multiply: args (&lhs, &rhs, &dest)
    Multiply,
    /// Terminate execution: no args
    Terminate,
}

impl OpCode {
    /// Indirect load returning a copy of the value
    fn get_at(ptr: usize, memory: &mut [u32]) -> Result<u32, Error> {
        Ok(*memory
            .get(*memory.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)?)
    }

    /// Indirect load returning mutable location reference
    fn get_mut_at(ptr: usize, memory: &mut [u32]) -> Result<&mut u32, Error> {
        Ok(memory
            .get_mut(*memory.get(ptr).ok_or(Error::MemoryError)? as usize)
            .ok_or(Error::MemoryError)?)
    }

    /// Execute the opcode at `ip`, returning the amount to increment `ip`
    pub fn execute(&self, ip: usize, memory: &mut [u32]) -> Result<usize, Error> {
        match self {
            OpCode::Add => {
                let left = OpCode::get_at(ip + 1, memory)?;
                let right = OpCode::get_at(ip + 2, memory)?;
                let result = left + right;
                *OpCode::get_mut_at(ip + 3, memory)? = result;
                Ok(4)
            }
            OpCode::Multiply => {
                *OpCode::get_mut_at(ip + 3, memory)? =
                    OpCode::get_at(ip + 1, memory)? * OpCode::get_at(ip + 2, memory)?;
                Ok(4)
            }
            OpCode::Terminate => Err(Error::Terminated),
        }
    }
}

impl TryFrom<u32> for OpCode {
    type Error = Error;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
        Ok(match i {
            1 => OpCode::Add,
            2 => OpCode::Multiply,
            99 => OpCode::Terminate,
            _ => Err(Error::InvalidInstruction)?,
        })
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
        match OpCode::try_from(memory[ip])?.execute(ip, memory) {
            Ok(d) => ip += d,
            Err(Error::Terminated) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
