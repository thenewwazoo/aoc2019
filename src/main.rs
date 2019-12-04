use aoc2019::*;

fn main() {
    println!("AOC 2019");
    println!(
        "day 1: {}",
        day1::run().unwrap_or_else(|e| format!("failure: {}", e))
    );
    println!(
        "day 2: {}",
        day2::run().unwrap_or_else(|e| format!("failure: {}", e))
    );
}
