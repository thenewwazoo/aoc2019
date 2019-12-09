use crate::day2::{Error, IndirectFetch, IndirectStore, OpCode};
use std::convert::From;
use std::num::ParseIntError;
use std::str::FromStr;

pub fn read<T>(prompt: &str) -> Result<T, Error>
where
    T: FromStr,
    ParseIntError: From<<T as FromStr>::Err>,
{
    print!("{} ", prompt);
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer)?;

    buffer
        .parse::<T>()
        .map_err(|e| Error::ParseIntError(e.into()))
}

struct Input;

impl OpCode for Input {
    fn execute(&self, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
        memory.store_at(ip + 3, read::<i32>("INPUT:")?)?;
        Ok(2)
    }
}

struct Output;

impl OpCode for Output {
    fn execute(&self, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
        println!("{}", memory.get_at(ip)?);
        Ok(2)
    }
}

// pub trait Store {}
// pub trait Fetch {}

// struct Immediate;
// struct Indirect;

#[cfg(test)]
mod test {}
