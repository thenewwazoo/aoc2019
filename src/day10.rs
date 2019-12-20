use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use num::integer::gcd;

type Point = (i64, i64);
type Ubermap = HashMap<Point, Map>;
type Map = HashSet<Point>;

pub fn find_best(map: &mut Ubermap) -> (Point, usize) {
    let nodes = map.keys().cloned().collect::<Vec<Point>>();
    let map_extent = get_map_extents(&map);
    print_map(&map);
    let mut k = 0;
    for node in nodes {
        for other in &map[&node].clone() {
            if &node == other {
                continue;
            }
            // get every point in the grid that's on the line described by the two points
            let ongrid_pts = get_grid_points_bw(node, *other, map_extent);
            //if ongrid_pts.len() < 2 {
                println!("ongrid {:?} -> {:?} : {:?}\n", &node, &other, &ongrid_pts);
            //}

            // find every spot on that line that's occupied
            let occ = find_occupied_points(&ongrid_pts, map);

            let occ_len = occ.len();
            if occ_len > 2 {
                println!("occ {:?}", &occ);
                for i in 0..occ_len {
                    let mut j = i + 2;
                    while let Some(m) = occ.get(j) {
                        map.get_mut(&occ[i]).unwrap().remove(m);
                        j += 1;
                    }

                    let mut j: isize = i as isize - 2;
                    while j >= 0 {
                        map.get_mut(&occ[i]).unwrap().remove(occ.get(j as usize).unwrap());
                        j -= 1;
                    }
                }
                //print_map(&map);
            }
            k += 1;
        }
    }
    let best = map.iter().max_by_key(|(_, v)| v.len()).unwrap();
    println!("k is {} of {}", k, map.len());
    (*best.0, best.1.len() - 1)
}

fn read_map_file(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(BufReader::new(File::open(filename)?)
        .lines()
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_map(lines: Vec<String>) -> Result<Ubermap, Box<dyn Error>> {
    let locs = lines
        .iter()
        .enumerate()
        .flat_map(|(i, l)| {
            l.bytes()
                .into_iter()
                .enumerate()
                .map(|(j, b)| {
                    if b == b'#' {
                        Some((j as i64, i as i64))
                    } else {
                        None
                    }
                })
                .filter(Option::is_some)
                .map(|p| p.unwrap())
                .collect::<Vec<Point>>()
        })
        .collect::<Vec<Point>>();
    let proto = locs.iter().cloned().collect::<HashSet<Point>>();
    Ok(locs.into_iter().map(|p| (p, proto.clone())).collect())
}

fn get_map_extents(map: &Ubermap) -> Point {
    (map.keys().map(|p| p.1).max().unwrap(), map.keys().map(|p| p.0).max().unwrap())
}

pub fn run() -> Result<String, Box<dyn Error>> {

    Ok(String::new())
}

fn get_grid_points_bw(origin: Point, end: Point, map_corner: Point) -> Vec<Point> {
    let rise = end.1 - origin.1;
    let run = end.0 - origin.0;
    let _gcd = gcd(rise, run);
    if _gcd == 0 {
        panic!("zero gcd for {} {}", rise, run);
    }
    let slope = Slope::from((rise / _gcd, run / _gcd));
    let inv_slope = slope.reverse();
    let mut pts: Vec<Point> = get_grid_points(
        origin.0,
        origin.1,
        if slope.rise > 0 {
            &slope
        } else {
            &inv_slope
        },
        map_corner.0,
        map_corner.1
    )
        .into_iter()
        .chain(
            get_grid_points(
                origin.0,
                origin.1,
                if slope.rise <= 0 {
                    &slope
                } else {
                    &inv_slope
                },
                map_corner.1,
                map_corner.1,
            )
            .into_iter()
        )
        .collect();
    pts.dedup();
    pts
}

fn get_grid_points(mut x: i64, mut y: i64, slope: &Slope, max_x: i64, max_y: i64) -> Vec<Point> {
    let mut pts = Vec::new();
    let mut loops = 0;
    println!("ggp from {}, {} w/ slope {:?}", x, y, slope);
    while x <= max_x && y <= max_y && x >= 0 && y >= 0 {
        println!("ggp at {}, {}", x, y);
        assert!(loops < 10);
        pts.push((x, y));
        y += slope.rise;
        x += slope.run;
        loops += 1;
    }
    pts.sort();
    pts
}

fn print_map(map: &Ubermap) {
    let (max_x, max_y) = get_map_extents(map);

    for y in 0..=max_y {
        for x in 0..=max_x {
            match map.get(&(x, y)) {
                Some(v) => print!("{:2} ", v.len()-1),
                None =>    print!(" . "),
            }
        }
        println!("");
    }
    println!("");
}

fn find_occupied_points(grid_pts: &[Point], map: &Ubermap) -> Vec<Point> {
    let mut s: Vec<Point> = grid_pts.iter().filter(|p| map.contains_key(p)).cloned().collect();
    s.sort_by_key(|p| p.0 + p.1);
    s
}

#[derive(Debug)]
struct Slope {
    pub rise: i64,
    pub run: i64,
}

impl Slope {
    pub fn reverse(&self) -> Self {
        Slope {
            rise: self.rise * -1,
            run: self.run * -1,
        }
    }
}

impl From<Point> for Slope {
    fn from((rise, run): Point) -> Self {
        Slope { rise, run }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[rustfmt::skip]
    const SM_DATA: &str = concat!(
        ".#..#\n",    //   o don't forget, y axis grows down
        ".....\n",    //   |
        "#####\n",    //   |
        "....#\n",    //   |
        "...##\n");   //   v

    #[rustfmt::skip]
    const M1_DATA: &str = concat!(
        "......#.#.\n",
        "#..#.#....\n",
        "..#######.\n",
        ".#.#.###..\n",
        ".#..#.....\n",
        "..#....#.#\n",
        "#..#....#.\n",
        ".##.#..###\n",
        "##...#..#.\n",
        ".#....####\n",
    );

    #[rustfmt::skip]
    const M2_DATA: &str = concat!(
        "#.#...#.#.\n",
        ".###....#.\n",
        ".#....#...\n",
        "##.#.#.#.#\n",
        "....#.#.#.\n",
        ".##..###.#\n",
        "..#...##..\n",
        "..##....##\n",
        "......#...\n",
        ".####.###.\n",
    );

    #[test]
    fn sm_data() {
        let mut map = parse_map(SM_DATA.lines().map(str::to_string).collect()).unwrap();
        let best = find_best(&mut map);
        print_map(&map);
        assert_eq!(best, ((3,4), 8));
    }

    #[test]
    fn m1_data() {
        let mut map = parse_map(M1_DATA.lines().map(str::to_string).collect()).unwrap();
        let best = find_best(&mut map);
        print_map(&map);
        assert_eq!(best, ((5,8), 33));
    }

    #[test]
    fn m2_data() {
        let mut map = parse_map(M2_DATA.lines().map(str::to_string).collect()).unwrap();
        let best = find_best(&mut map);
        print_map(&map);
        assert_eq!(best, ((1,2), 35));
    }

    #[test]
    fn find_occ_pts() {
        let map = parse_map(SM_DATA.lines().map(str::to_string).collect()).unwrap();
        let (max_x, max_y) = get_map_extents(&map);
        assert_eq!((max_x, max_y), (4, 4));

        assert_eq!(
            find_occupied_points(&get_grid_points(0, 0, &Slope{ rise: 1, run: 0 }, max_x, max_y), &map),
            vec![(0, 2)],
        );

        assert_eq!(
            find_occupied_points(&get_grid_points(0, 0, &Slope{ rise: 1, run: 1 }, max_x, max_y), &map),
            vec![(2,2), (4,4)],
        );

        assert_eq!(
            find_occupied_points(&get_grid_points(0, 1, &Slope{ rise: 0, run: 1 }, max_x, max_y), &map),
            vec![],
        );

        assert_eq!(
            find_occupied_points(&get_grid_points(0, 2, &Slope{ rise: 0, run: 1 }, max_x, max_y), &map),
            vec![(0,2), (1,2), (2,2), (3,2), (4,2)]
        );

        assert_eq!(
            find_occupied_points(&get_grid_points(0, 1, &Slope{ rise: 1, run: 2 }, max_x, max_y), &map),
            vec![(2,2), (4,3)],
        );
    }

    #[test]
    fn grid_points() {
        assert_eq!(
            get_grid_points(0, 0, &Slope { rise: 1, run: 1 }, 2, 2),
            vec![(0, 0), (1, 1), (2, 2)]
        );

        assert_eq!(
            get_grid_points(0, 0, &Slope { rise: 2, run: 1 }, 2, 2),
            vec![(0, 0), (1, 2)]
        );
    }

}
