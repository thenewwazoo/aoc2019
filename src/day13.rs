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
    Empty,
    Wall,
    Block,
    Paddle,
    Ball, // 4
    Score(i64),
}

impl Default for Tile {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<i64> for Tile {
    fn from(v: i64) -> Self {
        match v {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::Paddle,
            4 => Self::Ball,
            _ => Self::Score(v),
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Empty => ' ',
                Self::Wall => '+',
                Self::Ball => '•',
                Self::Block => '◊',
                Self::Paddle => '–',
                _ => panic!("cannot display score!"),
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HDir {
    L = -1,
    R = 1,
    S = 0,
}

impl HDir {
    pub fn flip(&mut self) {
        *self = match self {
            Self::L => Self::R,
            Self::R => Self::L,
            _ => Self::S,
        };
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VDir {
    U = -1,
    D = 1,
    S = 0,
}

impl VDir {
    pub fn flip(&mut self) {
        *self = match self {
            Self::U => Self::D,
            Self::D => Self::U,
            _ => Self::S,
        };
    }
}

#[derive(Clone)]
struct Ball {
    pub loc: Point,
    pub h: HDir,
    pub v: VDir,
}

impl std::fmt::Display for Ball {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "({}, {}) {} {}",
            self.loc.0,
            self.loc.1,
            match self.h {
                HDir::L => "«",
                HDir::R => "»",
                HDir::S => "|",
            },
            match self.v {
                VDir::U => "^",
                VDir::D => "v",
                VDir::S => "-",
            }
        )
    }
}

impl Default for Ball {
    fn default() -> Self {
        Ball {
            loc: (0, 0),
            h: HDir::S,
            v: VDir::S,
        }
    }
}

impl Ball {
    pub fn freeze(&mut self) {
        self.h = HDir::S;
        self.v = VDir::S;
    }

    pub fn update(&mut self, l: Point) {
        if l.0 == self.loc.0 {
            self.h = HDir::S;
        } else if l.1 == self.loc.1 {
            self.v = VDir::S;
        } else {
            if l.0 - self.loc.0 > 0 {
                self.h = HDir::R;
            } else {
                self.h = HDir::L;
            }
            if l.1 - self.loc.1 > 0 {
                self.v = VDir::D;
            } else {
                self.v = VDir::U;
            }
        }
        self.loc = l;
    }

    pub fn predict_landing(&self, h: i64, screen: &Map) -> Option<i64> {
        if self.h == HDir::S || self.v == VDir::S {
            return None;
        }

        let mut probe = self.clone();
        let mut screen = screen.clone();
        println!("probe is at {}", self);
        let mut loops = 0;
        while probe.loc.1 < h && probe.loc.1 >= 0 && probe.loc.0 >= 0 && probe.loc.0 < 50 {
            if let Some(t) = screen.get(&(probe.loc.0 + probe.h as i64, probe.loc.1)) {
                if t == &Tile::Wall {
                    println!("found wall <->");
                    probe.h.flip();
                }
                if t == &Tile::Block {
                    println!("found block <>");
                    probe.h.flip();
                    screen
                        .remove(&(probe.loc.0 + probe.h as i64, probe.loc.1))
                        .unwrap();
                }
            }
            if let Some(t) = screen.get(&(probe.loc.0, probe.loc.1 + probe.v as i64)) {
                if t == &Tile::Wall {
                    println!("found wall ^v");
                    probe.v.flip();
                }
                if t == &Tile::Block {
                    println!("found block ^v");
                    probe.v.flip();
                    screen
                        .remove(&(probe.loc.0, probe.loc.1 + probe.v as i64))
                        .unwrap();
                }
            }
            if let Some(t) =
                screen.get(&(probe.loc.0 + probe.h as i64, probe.loc.1 + probe.v as i64))
            {
                if t == &Tile::Wall {
                    println!("found wall x");
                    probe.v.flip();
                    probe.h.flip();
                }
                if t == &Tile::Block {
                    println!("found block x");
                    probe.h.flip();
                    probe.v.flip();
                    screen
                        .remove(&(probe.loc.0 + probe.h as i64, probe.loc.1 + probe.v as i64))
                        .unwrap();
                }
            }
            probe.update((probe.loc.0 + probe.h as i64, probe.loc.1 + probe.v as i64));
            println!("stepped probe to {}", probe);
            if loops > 50 {
                panic!("whut");
            }
            loops += 1;
        }

        if probe.loc.1 == h {
            Some(probe.loc.0)
        } else {
            None
        }
    }
}

pub fn run() -> Result<String, Error> {
    let mut screen: HashMap<Point, Tile> = HashMap::new();

    let mut data = read_comma_file("input/day13.txt")?;
    data[0] = 2;
    data.extend(vec![0; 4096]);

    let mut machine = build_machine(data);
    let (tx, rx) = channel();
    machine.wire_output(tx);
    let tx = machine.wire_input();

    let t_h = spawn(move || machine.run());

    let mut score;
    let mut ball: Option<Ball> = None;
    let mut paddle_loc = (21, 22);
    let mut last_known_dest = 21;

    loop {
        match (rx.recv(), rx.recv(), rx.recv()) {
            (Ok(x), Ok(y), Ok(tile)) => {
                let tile = if x == -1 && y == 0 {
                    Tile::Score(tile)
                } else {
                    let tile = tile.into();
                    screen.insert((x, y), tile);
                    tile
                };

                match tile {
                    Tile::Ball => match &mut ball {
                        None => {
                            println!("got new ball at {} {}", x, y);
                            let mut _ball = Ball::default();
                            _ball.update((x, y));
                            ball = Some(_ball);
                            print_map(&screen, false);
                            println!("");
                            continue;
                        }
                        Some(b) => {
                            b.update((x, y));
                            println!("ball moved to {}", b);
                        }
                    },
                    Tile::Score(s) => {
                        score = s;
                        println!("score is {}", score);
                        ball.as_mut().unwrap().freeze();
                        println!("ball frozen");
                    }
                    Tile::Paddle => {
                        if paddle_loc.0 == 0 {
                            paddle_loc = (x, y);
                        }
                        println!("paddle moved from {} to {}", paddle_loc.0, x);
                        print_map(&screen, false);
                        println!("");
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }

                print_map(&screen, false);

                /*
                if let Some(dest) = ball
                    .clone()
                    .unwrap()
                    .predict_landing(paddle_loc.1 - 1, &screen)
                {
                    last_known_dest = dest;
                }
                if last_known_dest == paddle_loc.0 {
                    // stay
                    println!("{} STAY for {}", &ball.clone().unwrap(), last_known_dest);
                    if let Err(_) = tx.send(0) {
                        error!("could not send STAY");
                    }
                } else if last_known_dest > paddle_loc.0 {
                    // right
                    println!("{} RIGHT for {}", &ball.clone().unwrap(), last_known_dest);
                    paddle_loc.0 += 1;
                    if let Err(_) = tx.send(1) {
                        error!("could not send RIGHT");
                    }
                } else {
                    // left
                    println!("{} LEFT for {}", &ball.clone().unwrap(), last_known_dest);
                    paddle_loc.0 -= 1;
                    if let Err(_) = tx.send(-1) {
                        error!("could not send LEFT");
                    }
                }
                */
                if let Err(_) = tx.send(0) {
                    error!("bad dog");
                }
                println!("");
            }
            _ => {
                println!("done");
                break;
            }
        };
    }

    let _ = t_h.join().unwrap();

    print_map(&screen, false);
    Ok(format!(
        "{}",
        screen.values().filter(|v| **v == Tile::Block).count()
    ))
}
