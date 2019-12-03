
pub mod day1 {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    pub use error::*;

    mod error {
        #[derive(Debug)]
        pub enum Error {
            File,
            IoError(std::io::Error),
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

    /// The Fuel Counter-Upper needs to know the total fuel requirement. To find it, individually
    /// calculate the fuel needed for the mass of each module (your puzzle input), then add
    /// together all the fuel values.
    ///
    /// What is the sum of the fuel requirements for all of the modules on your spacecraft?
    pub fn run() -> Result<String, Error> {
        let input_filename = "input/day1.txt";

        let values = BufReader::new(File::open(input_filename)?)
            .lines()
            .map(|l| u64::from_str_radix(&l?, 10).map_err(|e| e.into()))
            .collect::<Result<Vec<u64>, Error>>()?;
        let total: u64 = values.iter().map(|v| fuel_counter_upper(*v)).sum();

        Ok(format!("{}", total))
    }

    /// Fuel required to launch a given module is based on its mass. Specifically, to find the fuel
    /// required for a module, take its mass, divide by three, round down, and subtract 2.
    pub fn fuel_counter_upper(mass: u64) -> u64 {
        mass / 3 - 2
    }

    #[cfg(test)]
    mod tests {
        use super::fuel_counter_upper;

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
    }

}
