use crate::day2::{Error, OpCode, read_comma_input, IntCodeMachine};
use crate::day2::mem::{ParamMode, Store, Fetch};
use crate::day2::ops::*;
use std::convert::From;
use std::num::ParseIntError;
use std::str::FromStr;
use std::io::Write;
use std::io::stdout;

pub fn read<T>(prompt: &str) -> Result<T, Error>
where
    T: FromStr,
    ParseIntError: From<<T as FromStr>::Err>,
{
    print!("{} ", prompt);
    let mut buffer = String::new();
    let _ = stdout().flush();
    let _ = std::io::stdin().read_line(&mut buffer)?;

    buffer
        .trim_end()
        .parse::<T>()
        .map_err(|e| Error::ParseIntError(e.into()))
}

struct Input;

impl OpCode for Input {
    fn execute(&self, param_code: i32, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
        let params = decode(param_code, 2)?;
        assert!(params[1] != ParamMode::Immediate);
        memory.store(ip + 3, &params[0], read::<i32>("INPUT:")?)?;
        Ok(2)
    }
}

struct Output;

impl OpCode for Output {
    fn execute(&self, param_code: i32, ip: usize, memory: &mut [i32]) -> Result<usize, Error> {
        let params = decode(param_code, 2)?;
        println!("OUTPUT: {}", memory.fetch(ip, &params[0])?);
        Ok(2)
    }
}

pub fn run() -> Result<String, Error> {
    let instrs = read_comma_input("input/day5.txt")?;

    let mut machine = IntCodeMachine::boot(instrs);
    machine.register_opcode(3, Box::new(Input));
    machine.register_opcode(4, Box::new(Output));

    let _end_state = machine.run()?;
    Ok(format!("ok"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn param_test() {
        let mut mem = vec![1002, 4, 3, 4, 33];
        let op = Multiply;
        assert!(op.execute(10, 0, &mut mem).is_ok());
        assert_eq!(mem, vec![1002, 4, 3, 4, 99]);
    }

    #[test]
    fn negative_value() {
        let mut mem = vec![1101,100,-1,4,0];
        let op = Add;
        assert!(op.execute(11, 0, &mut mem).is_ok());
        assert_eq!(mem, vec![1101, 100, -1, 4, 99]);
    }
}
