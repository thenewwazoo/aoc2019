use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum Error {
    BadSplit,
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

fn read_file(filename: &str) -> Result<Vec<(String, String)>, Error> {
    BufReader::new(File::open(filename)?)
        .lines()
        .map(|l| {
            let l = l?;
            let v = l.split(")").collect::<Vec<&str>>();
            Ok((v[0].to_string(), v[1].to_string()))
        })
        .collect()
}

fn collect_pairs(pairs: &[(String, String)]) -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    for (ref k, ref v) in pairs.into_iter() {
        let l = m.entry(k.clone()).or_insert(Vec::new());
        l.push(v.clone());
    }
    m
}

fn count_depth(mut map: HashMap<String, Vec<String>>) -> usize {
    let mut stack = vec![String::from("COM")];
    let mut total = 0;
    while !stack.is_empty() {
        /*
        println!(
            "{}",
            stack
                .iter()
                .fold(String::new(), |acc, s| format!("{} {}", acc, s))
        );
        */
        if let Some(children) = map.get(stack.last().unwrap()) {
            stack.push(children[0].clone());
            continue;
        }
        total += stack.len() - 1;
        let leaf = stack.pop().unwrap();
        if stack.is_empty() {
            break;
        }
        let idx = match map.get(stack.last().unwrap()) {
            Some(l) => l.iter().position(|s| *s == leaf).unwrap(),
            None => {
                break;
            }
        };
        map.get_mut(stack.last().unwrap()).unwrap().remove(idx);
        if map.get(stack.last().unwrap()).unwrap().is_empty() {
            map.remove(stack.last().unwrap());
        }
    }
    total
}

fn seek_path(map: &HashMap<String, Vec<String>>, start: &str, finish: &str) -> Option<Vec<String>> {
    let mut stack = vec![String::from(start)];
    let mut map = map.clone();
    loop {
        /*
        println!(
            "{}",
            stack
                .iter()
                .fold(String::new(), |acc, s| format!("{} {}", acc, s))
        );
        */
        // if we're not at a leaf
        if let Some(children) = map.get(stack.last().unwrap()) {
            // if this node holds our destination, we can bail immediately
            if children.iter().position(|s| s == finish).is_some() {
                stack.push(String::from(finish));
                return Some(stack);
            }

            // keep looking
            stack.push(children[0].clone());
            continue;
        }

        let leaf = stack.pop().unwrap();
        if stack.is_empty() {
            return None;
        }

        let idx = match map.get(stack.last().unwrap()) {
            Some(l) => l.iter().position(|s| *s == leaf).unwrap(),
            None => {
                return None;
            }
        };
        map.get_mut(stack.last().unwrap()).unwrap().remove(idx);
        if map.get(stack.last().unwrap()).unwrap().is_empty() {
            map.remove(stack.last().unwrap());
        }
    }
}

fn find_root(left: &mut VecDeque<String>, right: &mut VecDeque<String>) {
    let mut last = String::new();
    while left.as_slices().0.first() == right.as_slices().0.first() {
        last = left.pop_front().unwrap();
        right.pop_front();
    }
    left.push_front(last.to_string());
    right.push_front(last.to_string());
}

pub fn run() -> Result<String, Error> {
    let pairs = read_file("input/day6.txt")?;
    let pairs = collect_pairs(&pairs);

    let you = seek_path(&pairs, "COM", "YOU");
    let santa = seek_path(&pairs, "COM", "SAN");
    let mut you = VecDeque::from(you.unwrap());
    let mut santa = VecDeque::from(santa.unwrap());
    find_root(&mut you, &mut santa);
    let num_xfer = you.len() - 2 + santa.len() - 2;

    Ok(format!("{} {}", count_depth(pairs), num_xfer))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn small() {
        let data = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
        ]
        .into_iter()
        .map(|l| {
            let p = l.split(")").map(|s| s.to_string()).collect::<Vec<String>>();
            (p[0].to_string(), p[1].to_string())
        })
        .collect::<Vec<(String, String)>>();
        assert_eq!(count_depth(collect_pairs(&data)), 42);
    }

    #[test]
    fn depth() {
        let map = vec![
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
            "I)SAN",
        ]
        .into_iter()
        .map(|l| {
            let p = l.split(")").map(|s| s.to_string()).collect::<Vec<String>>();
            (p[0].to_string(), p[1].to_string())
        })
        .collect::<Vec<(String, String)>>();
        let map = collect_pairs(&map);
        println!("seeking you");
        let you = seek_path(&map, "COM", "YOU");
        assert_eq!(
            you,
            Some(vec![
                "COM".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "E".to_string(),
                "J".to_string(),
                "K".to_string(),
                "YOU".to_string(),
            ])
        );
        println!("seeking santa");
        let santa = seek_path(&map, "COM", "SAN");
        assert_eq!(
            santa,
            Some(vec![
                "COM".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "I".to_string(),
                "SAN".to_string()
            ])
        );
        let mut you = VecDeque::from(you.unwrap());
        let mut santa = VecDeque::from(santa.unwrap());
        find_root(&mut you, &mut santa);
        assert_eq!(you.len() - 2 + santa.len() - 2, 4);
    }
}
