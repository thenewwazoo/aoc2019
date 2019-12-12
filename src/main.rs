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
    println!(
        "day 3: {}, {}",
        day3::run().unwrap_or_else(|e| format!("failure: {:?}", e)),
        day3::run_part2().unwrap_or_else(|e| format!("failure: {:?}", e))
    );
    println!(
        "day 4: {}",
        day4::run()
    );
    println!("day 5 starting:");
    println!("day 5 result: {:?}", day5::run());
}
