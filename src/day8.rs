use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};

const HEIGHT: usize = 6;
const WIDTH: usize = 25;

pub fn run() -> Result<String, Box<dyn Error>> {
    let layers: Vec<Vec<u8>> = BufReader::new(File::open("input/day8.txt")?)
        .bytes()
        .map(Result::unwrap)
        .filter(|s| *s >= b'0')
        .map(|b| b - b'0')
        .fold(vec![Vec::new()], |mut acc, b| {
            if acc.last().unwrap().len() == HEIGHT * WIDTH {
                acc.push(vec![b]);
            } else {
                acc.last_mut().unwrap().push(b);
            }
            acc
        });

    let (ones, twos): (Vec<u8>, Vec<u8>) = layers
        .clone()
        .iter()
        .fold(None, |acc, layer| {
            let current_count = layer.iter().filter(|b| **b == 0).count();
            match acc {
                Some((_, last_count)) => {
                    if current_count < last_count {
                        Some((layer, current_count))
                    } else {
                        acc
                    }
                }
                None => Some((layer, current_count)),
            }
        })
        .unwrap()
        .0
        .iter()
        .filter(|b| **b != 0)
        .partition(|b| **b == 1);

    let rendered: Vec<u8> = layers
        .iter()
        .fold(vec![2; HEIGHT * WIDTH], |acc, layer| {
            acc.iter()
                .zip(layer.iter())
                .map(|(top, bot)| if *top == 2 { *bot } else { *top })
                .collect()
        })
        .iter()
        .map(|b| if *b == 0 { b' ' } else { b'.' })
        .collect();

    // dbg!(&ones, &twos);
    Ok(format!(
        "{}\n{}",
        ones.len() * twos.len(),
        rendered.iter().cloned().map(char::from).enumerate().fold(
            String::new(),
            |mut acc, (c, pixel)| {
                acc = format!("{}{}", acc, pixel);
                if (c + 1) % WIDTH == 0 {
                    acc = format!("{}\n", acc);
                }
                acc
            }
        )
    ))
}
