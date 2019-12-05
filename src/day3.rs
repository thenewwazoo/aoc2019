use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub enum Line {
    Horiz {
        /// Y coordinate of the line
        y: i64,
        /// X coordinate of the left end
        l: i64,
        /// X coordinate of the right end
        r: i64,
    },
    Vert {
        /// X coordinate of the line
        x: i64,
        /// Y coordinate of the upper end
        u: i64,
        /// Y coordinate of the lower end
        d: i64,
    },
}

impl Line {
    pub fn intersects(&self, other: &Self) -> bool {
        match self {
            Line::Horiz { y, l, r } => match other {
                Line::Vert { x, u, d } => intersects(*y, *l, *r, *x, *u, *d),
                _ => false,
            },
            Line::Vert { x, u, d } => match other {
                Line::Horiz { y, l, r } => intersects(*y, *l, *r, *x, *u, *d),
                _ => false,
            },
        }
    }

    pub fn dist(&self, other: &Self) -> Result<i64, DirErr> {
        if !self.intersects(other) {
            return Err(DirErr::DoesNotIntersect);
        }

        match self {
            Line::Horiz { y, .. } => match other {
                Line::Vert { x, .. } => Ok(y.abs() + x.abs()),
                _ => Err(DirErr::DoesNotIntersect),
            },
            Line::Vert { x, .. } => match other {
                Line::Horiz { y, .. } => Ok(y.abs() + x.abs()),
                _ => Err(DirErr::DoesNotIntersect),
            },
        }
    }
}

/// h ∩ v iff l < x && r > x && y < u && y > d
fn intersects(y: i64, l: i64, r: i64, x: i64, u: i64, d: i64) -> bool {
    l < x && r > x && y < u && y > d
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    U(i64),
    D(i64),
    L(i64),
    R(i64),
}

#[derive(Debug, PartialEq)]
pub enum DirErr {
    ParseIntError(std::num::ParseIntError),
    IoError(std::io::ErrorKind),
    BadDir(String),
    DoesNotIntersect,
}

impl From<std::num::ParseIntError> for DirErr {
    fn from(e: std::num::ParseIntError) -> Self {
        DirErr::ParseIntError(e)
    }
}

impl From<std::io::Error> for DirErr {
    fn from(e: std::io::Error) -> Self {
        DirErr::IoError(e.kind())
    }
}

impl TryFrom<&str> for Direction {
    type Error = DirErr;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s.as_bytes()[0] {
            b'U' => Direction::U(s[1..].parse::<i64>()?),
            b'D' => Direction::D(s[1..].parse::<i64>()?),
            b'L' => Direction::L(s[1..].parse::<i64>()?),
            b'R' => Direction::R(s[1..].parse::<i64>()?),
            _ => Err(DirErr::BadDir(s.to_string()))?,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Cursor(i64, i64);

impl Cursor {
    pub fn new() -> Self {
        Cursor(0, 0)
    }

    pub fn mov(&mut self, dir: &Direction) -> Line {
        match dir {
            Direction::U(m) => {
                let line = Line::Vert {
                    x: self.0,
                    d: self.1,
                    u: self.1 + m,
                };
                self.1 += m;
                line
            }
            Direction::D(m) => {
                let line = Line::Vert {
                    x: self.0,
                    u: self.1,
                    d: self.1 - m,
                };
                self.1 -= m;
                line
            }
            Direction::L(m) => {
                let line = Line::Horiz {
                    y: self.1,
                    r: self.0,
                    l: self.0 - m,
                };
                self.0 -= m;
                line
            }
            Direction::R(m) => {
                let line = Line::Horiz {
                    y: self.1,
                    l: self.0,
                    r: self.0 + m,
                };
                self.0 += m;
                line
            }
        }
    }
}

fn gather_segments(seq: &[Direction]) -> (Vec<Line>, Vec<Line>) {
    let mut cursor = Cursor::new();
    seq.iter().map(|m| cursor.mov(m)).partition(|l| match l {
        Line::Vert { .. } => true,
        Line::Horiz { .. } => false,
    })
}

fn read_directions(filename: &str) -> Result<[Vec<Direction>; 2], DirErr> {
    let mut r = BufReader::new(File::open(filename)?)
        .lines()
        .map(|l| {
            l?.split(",")
                .map(|s| Direction::try_from(s))
                .collect::<Result<Vec<Direction>, DirErr>>()
        })
        .collect::<Vec<Result<Vec<Direction>, DirErr>>>();
    let s = r.pop().unwrap()?;
    let f = r.pop().unwrap()?;
    Ok([f, s])
}

pub fn run() -> Result<String, DirErr> {
    let dir_lists = read_directions("input/day3.txt")?;
    let (line1_h, line1_v) = gather_segments(&dir_lists[0]);
    let (line2_h, line2_v) = gather_segments(&dir_lists[1]);

    Ok(format!(
        "{}",
        std::cmp::min(
            find_closest_intersection(line1_h, line2_v)?,
            find_closest_intersection(line2_h, line1_v)?,
        )
    ))
}

fn find_closest_intersection(line_h: Vec<Line>, line_v: Vec<Line>) -> Result<i64, DirErr> {
    let mut min = 0x7FFF_FFFF_FFFF_FFFF;
    for h in line_h {
        for v in &line_v {
            if h.intersects(v) {
                let d = h.dist(v)?;
                if d < min {
                    min = d;
                }
            } else {
                continue;
            }
        }
    }
    Ok(min)
}

#[cfg(test)]
mod test {
    use super::*;

    const TOP: Line = Line::Horiz { y: 1, l: -2, r: 2 };
    const BOT: Line = Line::Horiz { y: -1, l: -2, r: 2};
    const LFT: Line = Line::Vert { x: -1, u: 2, d: -2 };
    const RGT: Line = Line::Vert { x: 1, u: 2, d: -2 };

    const UP: Direction = Direction::U(1);
    const DN: Direction = Direction::D(1);
    const LT: Direction = Direction::L(1);
    const RT: Direction = Direction::R(1);

    #[test]
    fn intersects() {
        assert!(TOP.intersects(&LFT));
        assert!(LFT.intersects(&TOP));

        assert!(TOP.intersects(&RGT));
        assert!(RGT.intersects(&TOP));

        assert!(BOT.intersects(&RGT));
        assert!(RGT.intersects(&BOT));

        assert!(BOT.intersects(&LFT));
        assert!(LFT.intersects(&BOT));

        assert!(!LFT.intersects(&RGT));
        assert!(!RGT.intersects(&LFT));
        assert!(!TOP.intersects(&BOT));
        assert!(!BOT.intersects(&TOP));

    }

    #[test]
    fn dist() {
        assert_eq!(TOP.dist(&LFT), Ok(2));
        assert_eq!(LFT.dist(&TOP), Ok(2));

        assert_eq!(BOT.dist(&LFT), Ok(2));
        assert_eq!(LFT.dist(&BOT), Ok(2));

        assert_eq!(TOP.dist(&RGT), Ok(2));
        assert_eq!(RGT.dist(&TOP), Ok(2));

        assert_eq!(BOT.dist(&RGT), Ok(2));
        assert_eq!(RGT.dist(&BOT), Ok(2));

        assert_eq!(TOP.dist(&BOT), Err(DirErr::DoesNotIntersect));
        assert_eq!(BOT.dist(&TOP), Err(DirErr::DoesNotIntersect));
        assert_eq!(LFT.dist(&RGT), Err(DirErr::DoesNotIntersect));
        assert_eq!(RGT.dist(&LFT), Err(DirErr::DoesNotIntersect));

    }

    #[test]
    fn make_dir() {
        assert_eq!(Direction::try_from("U1"), Ok(UP));
        assert_eq!(Direction::try_from("D1"), Ok(DN));
        assert_eq!(Direction::try_from("L1"), Ok(LT));
        assert_eq!(Direction::try_from("R1"), Ok(RT));
        assert_eq!(
            Direction::try_from("bad"),
            Err(DirErr::BadDir("bad".to_string()))
        );
    }

    #[test]
    fn turtle() {
        let mut c = Cursor::new();
        c.mov(&UP);
        assert_eq!(c, Cursor(0, 1));
        c.mov(&RT);
        assert_eq!(c, Cursor(1, 1));
        c.mov(&DN);
        assert_eq!(c, Cursor(1, 0));
        c.mov(&LT);
        assert_eq!(c, Cursor(0, 0));
        c.mov(&Direction::U(0)); // move zero distance
        assert_eq!(c, Cursor::new());
    }
}


