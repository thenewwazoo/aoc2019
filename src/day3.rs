use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};

type RunningLine = (u64, Line);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Horiz {
        /// Y coordinate of the line
        y: i64,
        /// X coordinate of the left end
        l: i64,
        /// X coordinate of the right end
        r: i64,
        /// The direction vector
        h: Direction,
    },
    Vert {
        /// X coordinate of the line
        x: i64,
        /// Y coordinate of the upper end
        u: i64,
        /// Y coordinate of the lower end
        d: i64,
        /// Direction vector
        v: Direction,
    },
}

impl Line {
    pub fn intersects(&self, other: &Self) -> bool {
        match self {
            Line::Horiz { y, l, r, .. } => match other {
                Line::Vert { x, u, d, .. } => intersects(*y, *l, *r, *x, *u, *d),
                _ => false,
            },
            Line::Vert { x, u, d, .. } => match other {
                Line::Horiz { y, l, r, .. } => intersects(*y, *l, *r, *x, *u, *d),
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

    pub fn intersection_depth(&self, other: &Self) -> Result<i64, DirErr> {
        if !self.intersects(other) {
            return Err(DirErr::DoesNotIntersect);
        }

        // make it so we can know which is which, in order to enable matching in the next stanza
        let (horiz, vert) = match self {
            Line::Horiz { .. } => (&self, &other),
            Line::Vert { .. } => (&other, &self),
        };

        match (*horiz, *vert) {
            (
                Line::Horiz {
                    y,
                    r,
                    h: Direction::L(_),
                    ..
                },
                Line::Vert {
                    x,
                    d,
                    v: Direction::U(_),
                    ..
                },
            ) => {
                // leftward crossing upward
                Ok(r - x + y - d)
            }
            (
                Line::Horiz {
                    y,
                    r,
                    h: Direction::L(_),
                    ..
                },
                Line::Vert {
                    x,
                    u,
                    v: Direction::D(_),
                    ..
                },
            ) => {
                // leftward crossing downward
                Ok(r - x + u - y)
            }
            (
                Line::Horiz {
                    y,
                    l,
                    h: Direction::R(_),
                    ..
                },
                Line::Vert {
                    x,
                    d,
                    v: Direction::U(_),
                    ..
                },
            ) => {
                // rightward crossing upward
                Ok(x - l + y - d)
            }
            (
                Line::Horiz {
                    y,
                    l,
                    h: Direction::R(_),
                    ..
                },
                Line::Vert {
                    x,
                    u,
                    v: Direction::D(_),
                    ..
                },
            ) => {
                // rightward crossing downward
                Ok(x - l + u - y)
            }
            _ => unreachable!(),
        }
    }
}

/// h ∩ v iff l < x && r > x && y < u && y > d
fn intersects(y: i64, l: i64, r: i64, x: i64, u: i64, d: i64) -> bool {
    l < x && r > x && y < u && y > d
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    U(i64),
    D(i64),
    L(i64),
    R(i64),
}

impl Direction {
    pub fn length(&self) -> i64 {
        match self {
            Direction::U(d) => *d,
            Direction::D(d) => *d,
            Direction::L(d) => *d,
            Direction::R(d) => *d,
        }
    }
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
                    v: *dir,
                };
                self.1 += m;
                line
            }
            Direction::D(m) => {
                let line = Line::Vert {
                    x: self.0,
                    u: self.1,
                    d: self.1 - m,
                    v: *dir,
                };
                self.1 -= m;
                line
            }
            Direction::L(m) => {
                let line = Line::Horiz {
                    y: self.1,
                    r: self.0,
                    l: self.0 - m,
                    h: *dir,
                };
                self.0 -= m;
                line
            }
            Direction::R(m) => {
                let line = Line::Horiz {
                    y: self.1,
                    l: self.0,
                    r: self.0 + m,
                    h: *dir,
                };
                self.0 += m;
                line
            }
        }
    }
}

fn walk(seq: &[Direction]) -> Vec<Line> {
    let mut cursor = Cursor::new();
    seq.iter().map(|m| cursor.mov(m)).collect()
}

fn measured_walk(seq: &[Direction]) -> Vec<RunningLine> {
    let mut cursor = Cursor::new();
    let mut running_dist: u64 = 0;
    seq.iter()
        .map(|m| {
            let result = (running_dist, cursor.mov(m));
            running_dist += m.length() as u64;
            result
        })
        .collect()
}

/// Walks a sequence of Directions, returning two lists of the lines made
fn gather_segments(seq: &[Line]) -> (Vec<Line>, Vec<Line>) {
    seq.iter().partition(|l| match l {
        Line::Vert { .. } => true,
        Line::Horiz { .. } => false,
    })
}

fn gather_measured_segments(seq: &[RunningLine]) -> (Vec<RunningLine>, Vec<RunningLine>) {
    seq.iter().partition(|l| match l.1 {
        Line::Vert { .. } => true,
        Line::Horiz { .. } => false,
    })
}

fn dir_from_str(s: &str) -> Result<Vec<Direction>, DirErr> {
    s.split(",").map(|s| Direction::try_from(s)).collect()
}

fn read_directions(filename: &str) -> Result<[Vec<Direction>; 2], DirErr> {
    let mut r = BufReader::new(File::open(filename)?)
        .lines()
        .map(|l| dir_from_str(&l?))
        .collect::<Vec<Result<Vec<Direction>, DirErr>>>();
    let s = r.pop().unwrap()?;
    let f = r.pop().unwrap()?;
    Ok([f, s])
}

pub fn run() -> Result<String, DirErr> {
    let dir_lists = read_directions("input/day3.txt")?;
    let (line1_h, line1_v) = gather_segments(walk(&dir_lists[0]).as_slice());
    let (line2_h, line2_v) = gather_segments(walk(&dir_lists[1]).as_slice());

    Ok(format!(
        "{}",
        std::cmp::min(
            find_closest_intersection(&line1_h, &line2_v)?,
            find_closest_intersection(&line2_h, &line1_v)?,
        )
    ))
}

pub fn run_part2() -> Result<String, DirErr> {
    let dir_lists = read_directions("input/day3.txt")?;

    let first_path = measured_walk(&dir_lists[0]);
    let second_path = measured_walk(&dir_lists[1]);

    let (f_h, f_v) = gather_measured_segments(first_path.as_slice());
    let (s_h, s_v) = gather_measured_segments(second_path.as_slice());

    Ok(format!(
        "{}",
        std::cmp::min(
            find_nearest_intersection(&f_h, &s_v)?,
            find_nearest_intersection(&s_h, &f_v)?,
        )
    ))
}

fn find_closest_intersection(line_h: &[Line], line_v: &[Line]) -> Result<i64, DirErr> {
    let mut min = 0x7FFF_FFFF_FFFF_FFFF;
    for h in line_h {
        for v in line_v {
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

fn find_nearest_intersection(
    line_h: &[RunningLine],
    line_v: &[RunningLine],
) -> Result<u64, DirErr> {
    let mut min = 0x8000_0000_0000_0000;
    for h in line_h {
        for v in line_v {
            if h.1.intersects(&v.1) {
                // cast OK because intersection_depth is always positive
                let d = h.0 + v.0 + h.1.intersection_depth(&v.1)? as u64;
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

    const TOP: Line = Line::Horiz {
        y: 1,
        l: -2,
        r: 2,
        h: Direction::L(0),
    };
    const BOT: Line = Line::Horiz {
        y: -1,
        l: -2,
        r: 2,
        h: Direction::R(0),
    };
    const LFT: Line = Line::Vert {
        x: -1,
        u: 2,
        d: -2,
        v: Direction::U(0),
    };
    const RGT: Line = Line::Vert {
        x: 1,
        u: 2,
        d: -2,
        v: Direction::D(0),
    };

    /*
     * -2-1 0 1 2
     * .......^...
     * ...|...|...  2
     * ...|...|...
     * <--+---+--.  1
     * ...|...|...
     * ...|.o.|...  0
     * ...|...|...
     * .--+---+--> -1
     * ...|...|...
     * ...|...|... -2
     * ...v.......
     * ...........
     */

    const UP: Direction = Direction::U(1);
    const DN: Direction = Direction::D(1);
    const LT: Direction = Direction::L(1);
    const RT: Direction = Direction::R(1);

    const DRAW: &[Direction] = &[UP, RT, DN, LT];
    const SQUARE: &[Line] = &[
        Line::Vert {
            x: 0,
            d: 0,
            u: 1,
            v: UP,
        },
        Line::Horiz {
            y: 1,
            l: 0,
            r: 1,
            h: RT,
        },
        Line::Vert {
            x: 1,
            d: 0,
            u: 1,
            v: DN,
        },
        Line::Horiz {
            y: 0,
            l: 0,
            r: 1,
            h: LT,
        },
    ];

    #[test]
    fn isec_dists() {
        assert_eq!(TOP.intersection_depth(&LFT), Ok(6));
        assert_eq!(BOT.intersection_depth(&LFT), Ok(2));
        assert_eq!(TOP.intersection_depth(&RGT), Ok(2));
        assert_eq!(BOT.intersection_depth(&RGT), Ok(6));
    }

    #[test]
    fn gathering() {
        assert_eq!(
            gather_segments(walk(DRAW).as_slice()),
            (
                vec![SQUARE[0].clone(), SQUARE[2].clone()],
                vec![SQUARE[1].clone(), SQUARE[3].clone()]
            )
        );
    }

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

    #[test]
    fn intersection_1() {
        let first = dir_from_str("R75,D30,R83,U83,L12,D49,R71,U7,L72").unwrap();
        let second = dir_from_str("U62,R66,U55,R34,D71,R55,D58,R83").unwrap();
        let (fh, fv) = gather_segments(walk(&first).as_slice());
        let (sh, sv) = gather_segments(walk(&second).as_slice());
        assert_eq!(
            std::cmp::min(
                find_closest_intersection(&fh, &sv).unwrap(),
                find_closest_intersection(&sh, &fv).unwrap(),
            ),
            159
        );
    }

    #[test]
    fn intersection_2() {
        let first = dir_from_str("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51").unwrap();
        let second = dir_from_str("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").unwrap();
        let (fh, fv) = gather_segments(walk(&first).as_slice());
        let (sh, sv) = gather_segments(walk(&second).as_slice());
        assert_eq!(
            std::cmp::min(
                find_closest_intersection(&fh, &sv).unwrap(),
                find_closest_intersection(&sh, &fv).unwrap(),
            ),
            135
        );
    }
}
