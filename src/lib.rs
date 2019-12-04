/// AoC Day 1
pub mod day1 {
    pub use error::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    /// Error conditions related to day1
    mod error {
        #[derive(Debug)]
        /// Day 1-related error conditions
        pub enum Error {
            /// Error reading or opening file
            IoError(std::io::Error),
            /// Unable to parse int
            ParseIntError(std::num::ParseIntError),
        }

        impl From<std::io::Error> for Error {
            fn from(e: std::io::Error) -> Self {
                Error::IoError(e)
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
    }

    /// Generate output for day 1, both parts
    pub fn run() -> Result<String, Error> {
        let total_fuel = total_fuel("input/day1.txt")?;

        Ok(format!("{}", total_fuel))
    }

    /// The Fuel Counter-Upper needs to know the total fuel requirement.
    ///
    /// To find it, individually calculate the fuel needed for the mass of each module (your puzzle
    /// input), then add together all the fuel values.
    /// ...
    /// So, for each module mass, calculate its fuel and add it to the total. Then, treat the fuel
    /// amount you just calculated as the input mass and repeat the process, continuing until a
    /// fuel requirement is zero or negative.
    ///
    /// What is the sum of the fuel requirements for all of the modules on your spacecraft?
    pub fn total_fuel(input_filename: &str) -> Result<u64, Error> {
        Ok(BufReader::new(File::open(input_filename)?)
            .lines()
            .map(|l| u64::from_str_radix(&l?, 10).map_err(|e| e.into()))
            .collect::<Result<Vec<u64>, Error>>()?
            .iter()
            .map(|&v| fuel_counter_upper(v))
            .map(|v| v + tyrannical_fcu(v))
            .sum())
    }

    /// Rocket-Equation compensator
    ///
    /// Fuel itself requires fuel just like a module - take its mass, divide by three, round down,
    /// and subtract 2. However, that fuel also requires fuel, and that fuel requires fuel, and so
    /// on. Any mass that would require negative fuel should instead be treated as if it requires
    /// zero fuel; the remaining mass, if any, is instead handled by wishing really hard, which has
    /// no mass and is outside the scope of this calculation.
    pub fn tyrannical_fcu(mass: u64) -> u64 {
        let mut sum = 0;
        let mut d = fuel_counter_upper(mass);
        while d > 0 {
            sum += d;
            d = fuel_counter_upper(d);
        }
        sum
    }

    /// Fuel required to launch a given module is based on its mass. Specifically, to find the fuel
    /// required for a module, take its mass, divide by three, round down, and subtract 2.
    pub fn fuel_counter_upper(mass: u64) -> u64 {
        (mass / 3).saturating_sub(2)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// For example:
        ///
        /// For a mass of 12, divide by 3 and round down to get 4, then subtract 2 to get 2.
        /// For a mass of 14, dividing by 3 and rounding down still yields 4, so the fuel required is also 2.
        /// For a mass of 1969, the fuel required is 654.
        /// For a mass of 100756, the fuel required is 33583.
        fn fcu() {
            assert_eq!(fuel_counter_upper(12), 2);
            assert_eq!(fuel_counter_upper(14), 2);
            assert_eq!(fuel_counter_upper(1969), 654);
            assert_eq!(fuel_counter_upper(100756), 33583);
        }

        #[test]
        /// A module of mass 14 requires 2 fuel. This fuel requires no further fuel (2 divided by 3
        /// and rounded down is 0, which would call for a negative fuel), so the total fuel
        /// required is still just 2.
        /// At first, a module of mass 1969 requires 654 fuel. Then, this fuel requires 216 more
        /// fuel (654 / 3 - 2). 216 then requires 70 more fuel, which requires 21 fuel, which
        /// requires 5 fuel, which requires no further fuel. So, the total fuel required for a
        /// module of mass 1969 is 654 + 216 + 70 + 21 + 5 = 966.
        /// The fuel required by a module of mass 100756 and its fuel is: 33583 + 11192 + 3728 +
        /// 1240 + 411 + 135 + 43 + 12 + 2 = 50346.
        fn test_tyranny() {
            assert_eq!(tyrannical_fcu(14), 2);
            assert_eq!(tyrannical_fcu(1969), 966);
            assert_eq!(tyrannical_fcu(100756), 50346);
        }
    }
}

/// AoC Day 2
pub mod day2 {
    use std::convert::TryFrom;
    use std::io::{BufRead, BufReader};
    use std::fs::File;

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
        let mut instrs = BufReader::new(File::open("input/day2.txt")?)
            .split(b',')
            .map(|s| std::str::from_utf8(&s.unwrap()).unwrap().trim_end().to_string())
            .map(|s| u32::from_str_radix(&s, 10).map_err(|e| e.into()))
            .collect::<Result<Vec<u32>, Error>>()?;

        instrs[1] = 12;
        instrs[2] = 2;

        execute(&mut instrs)?;

        Ok(format!("{}", instrs[0]))
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
    mod tests {
        use super::execute;
        use super::OpCode;

        #[test]
        fn test_execute() {
            let mut program = vec![1, 0, 0, 0, 99];
            assert_eq!(execute(&mut program), Ok(()));
            assert_eq!(program, vec![2, 0, 0, 0, 99]);

            let mut program = vec![2,3,0,3,99];
            assert_eq!(execute(&mut program), Ok(()));
            assert_eq!(program, vec![2,3,0,6,99]);

            let mut program = vec![2,4,4,5,99,0];
            assert_eq!(execute(&mut program), Ok(()));
            assert_eq!(program, vec![2,4,4,5,99,9801]);

            let mut program = vec![1,1,1,4,99,5,6,0,99];
            assert_eq!(execute(&mut program), Ok(()));
            assert_eq!(program, vec![30,1,1,4,2,5,6,0,99]);
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
    }
}
