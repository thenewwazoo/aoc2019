use aoc2019::*;
use std::env::args;

fn main() {
    println!("AOC 2019");
    match args().nth(1).expect("usage: aoc2019 <num>").as_str() {
        "1" => println!(
            "day 1: {}",
            day1::run().unwrap_or_else(|e| format!("failure: {}", e))
        ),
        "2" => println!(
            "day 2: {}",
            day2::run().unwrap_or_else(|e| format!("failure: {}", e))
        ),
        "3" => println!(
            "day 3: {}, {}",
            day3::run().unwrap_or_else(|e| format!("failure: {:?}", e)),
            day3::run_part2().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        "4" => println!("day 4: {}", day4::run()),
        "5" => {
            println!("day 5 starting...");
            day5::run().unwrap();
        }
        "6" => println!(
            "day 6: {}",
            day6::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        "7" => println!(
            "day 7: {}",
            day7::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        "8" => println!(
            "day 8: {}",
            day8::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        "9" => println!(
            "day 9: {}",
            day9::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        /*
        "10" => println!(
            "day 10: {}",
            day10::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        */
        "11" => println!(
            "day 11: {}",
            day11::run().unwrap_or_else(|e| format!("failure: {:?}", e))
        ),
        _ => unimplemented!(),
    }
}
