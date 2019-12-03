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
