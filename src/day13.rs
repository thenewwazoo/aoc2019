use std::collections::HashMap;
use std::thread::spawn;
use std::sync::mpsc::channel;

use crate::day2::Error;
use crate::day2::read_comma_file;
use crate::day9::build_machine;
use crate::day11::{print_map, Point};


pub fn run() -> Result<String, Error> {

    let mut screen: HashMap<Point, i64> = HashMap::new();

    let mut data = read_comma_file("input/day13.txt")?;
    data.extend(vec![0; 1024]);

    let mut machine = build_machine(data);
    let (tx, rx) = channel();
    machine.wire_output(tx);

    let t_h = spawn(move || machine.run());

    loop {
        match (rx.recv(), rx.recv(), rx.recv()) {
            (Ok(x), Ok(y), Ok(tile)) => screen.insert((x, y), tile),
            _ => {
                println!("bleh");
                break;
            }
        };
    }

    let _ = t_h.join().unwrap();

    print_map(&screen, false);
    Ok(format!("{}", screen.values().filter(|v| **v == 2).count()))
}
