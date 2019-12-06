
pub fn run() -> String {
    const START: usize = 235741;
    const END: usize = 706948;

    let part1 = (START..END).map(usize::into_digits).filter(assess).count();
    let part2 = (START..END).map(usize::into_digits).filter(assess).filter(p2_counter).count();

    format!("{}, {}", part1, part2)
}

fn assess(num: &[u8; 6]) -> bool {
    let mut idx = 1;
    let mut doubled = false;
    while idx < 6 {
        if num[idx] == num[idx-1] {
            doubled = true;
        }
        if num[idx] < num[idx-1] {
            return false;
        }
        idx += 1;
    }
    doubled
}

fn p2_counter(num: &[u8; 6]) -> bool {
    use std::collections::HashMap;
    let mut fmap = HashMap::new();
    for i in num.iter() {
        let p = fmap.entry(i).or_insert(0);
        *p += 1;
    }
    fmap.values().any(|&n| n == 2)
}

pub trait Digits {
    fn into_digits(self) -> [u8; 6];
}

impl Digits for usize {
    fn into_digits(self) -> [u8; 6] {
        let s = format!("{:06}", self);
        let r = s.as_bytes();
        [
            r[0] - b'0',
            r[1] - b'0',
            r[2] - b'0',
            r[3] - b'0',
            r[4] - b'0',
            r[5] - b'0',
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn into_digits() {
        assert_eq!(123456usize.into_digits(), [1,2,3,4,5,6]);
        assert_eq!(023456usize.into_digits(), [0,2,3,4,5,6]);
        assert_eq!(456usize.into_digits(), [0,0,0,4,5,6]);
        assert_eq!(0usize.into_digits(), [0,0,0,0,0,0]);
    }

    #[test]
    fn assessor() {
        assert_eq!(assess(&111111usize.into_digits()), true);
        assert_eq!(assess(&223450usize.into_digits()), false);
        assert_eq!(assess(&123789usize.into_digits()), false);
    }

    #[test]
    fn p2_fcount() {
        assert_eq!(p2_counter(&112233usize.into_digits()), true);
        assert_eq!(p2_counter(&123444usize.into_digits()), false);
        assert_eq!(p2_counter(&111122usize.into_digits()), true);
    }

}
