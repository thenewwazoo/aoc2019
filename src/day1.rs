pub use error::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Error conditions related to day1
mod error;

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
mod tests;
