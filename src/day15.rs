use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread::spawn;

use crate::day11::{print_map, Point};
use crate::day2::read_comma_file;
use crate::day2::Error;
use crate::day9::build_machine;

type Map = HashMap<Point, Tile>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Tile {
    Start,
    Wall,
    Empty,
    Oxygen,
    Visited,
}

impl Default for Tile {
    fn default() -> Self {
        Self::Empty
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Start => 'Ø',
                Self::Wall => '+',
                Self::Empty => ' ',
                Self::Oxygen => '',
                Self::Visited => '•',
            }
        )
    }
}

#[derive(Clone, Copy, Debug)]
enum Card {
    N = 1,
    S,
    W,
    E,
}

fn move_pt(l: Point, d: Card) -> Point {
    match d {
        Card::N => (l.0, l.1 + 1),
        Card::S => (l.0, l.1 - 1),
        Card::W => (l.0 - 1, l.1),
        Card::E => (l.0 + 1, l.1),
    }
}

pub fn run() -> Result<String, Error> {
    let mut map: Map = HashMap::new();

    let mut data = read_comma_file("input/day15.txt")?;
    data.extend(vec![0; 4096]);

    let mut machine = build_machine(data);
    let (tx, rx) = channel();
    machine.wire_output(tx);
    let tx = machine.wire_input();

    let _t_h = spawn(move || machine.run());

    let mut loc = (0, 0);
    let mut next_cmd = Card::N;
    map.insert((0, 0), Tile::Start);

    let mut visited_locs = vec![loc];

    let mut best_path: Option<Vec<Point>> = None;

    while visited_locs.len() > 0 {
        println!("sending cmd {:?}", next_cmd);
        tx.send(next_cmd as i64).unwrap();
        if let Ok(status) = rx.recv() {
            println!("got response {}", status);
            match status {
                0 => {
                    map.insert(move_pt(loc, next_cmd), Tile::Wall);
                    next_cmd = match which_next(loc, &map) {
                        None => dir_from(loc, visited_locs.pop().unwrap()), // backtrack
                        Some(d) => d,
                    };
                }
                1 => {
                    loc = move_pt(loc, next_cmd);
                    map.insert(loc, Tile::Visited);
                    next_cmd = match which_next(loc, &map) {
                        None => dir_from(loc, visited_locs.pop().unwrap()),
                        Some(d) => {
                            visited_locs.push(loc);
                            d
                        }
                    };
                }
                2 => {
                    if best_path.is_some() {
                        if visited_locs.len() < best_path.as_ref().unwrap().len() {
                            best_path = Some(visited_locs.clone());
                        }
                    } else {
                        best_path = Some(visited_locs.clone());
                    }
                    loc = move_pt(loc, next_cmd);
                    next_cmd = dir_from(loc, visited_locs.pop().unwrap()); // backtrack
                }
                _ => panic!("got invalid response"),
            }
        }
    }

    print_map(&map, false);
    println!("{:?}", best_path);

    Ok(String::new())
}

fn which_next(loc: Point, map: &Map) -> Option<Card> {
    let mut cmd = Card::N; // try north first
    let mut tries = 0;
    while let Some(_) = map.get(&move_pt(loc, cmd)) {
        cmd = match &cmd {
            Card::N => Card::E,
            Card::E => Card::S,
            Card::S => Card::W,
            Card::W => Card::N,
        };
        tries += 1;
        if tries == 4 {
            break;
        }
    }
    if tries == 4 {
        None
    } else {
        Some(cmd)
    }
}

// to get FROM from TO to, move Card
fn dir_from(from: Point, to: Point) -> Card {
    println!("backtracking to {:?}", to);
    if from.0 == to.0 {
        if from.1 < to.1 {
            Card::N
        } else {
            Card::S
        }
    } else if from.0 < to.0 {
        Card::E
    } else {
        Card::W
    }
}
