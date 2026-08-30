use std::error::Error;

use std::str::FromStr;

#[derive(Clone, Debug)]
pub(crate) struct CrossBox {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) id: usize,
    pub(crate) len: usize,
    pub(crate) dir: Direction,
}

impl FromStr for CrossBox {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_ascii_whitespace();
        let id = split.next().unwrap().parse().unwrap();
        let len = split.next().unwrap().parse().unwrap();
        let dir = split.next().unwrap().parse().unwrap();

        Ok(CrossBox {
            x: 0,
            y: 0,
            id,
            len,
            dir,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "d" => Direction::Down,
            "r" => Direction::Right,
            "u" => Direction::Up,
            "l" => Direction::Left,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Crossing {
    pub(crate) id_a: usize,
    pub(crate) id_b: usize,
    pub(crate) c_a: i32,
    pub(crate) c_b: i32,
}

impl FromStr for Crossing {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_ascii_whitespace();
        let id_a = split.next().unwrap().parse().unwrap();
        let id_b = split.next().unwrap().parse().unwrap();
        let c_a = split.next().unwrap().parse().unwrap();
        let c_b = split.next().unwrap().parse().unwrap();

        Ok(Crossing {
            id_a,
            id_b,
            c_a,
            c_b,
        })
    }
}

pub(crate) fn umlaut(input: &str) -> String {
    let input = input.to_ascii_lowercase();
    input
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

pub fn parse_file(
    b: &str,
    c: &str,
) -> Result<(Vec<CrossBox>, Vec<Crossing>), Box<dyn Error + 'static>> {
    let mut boxes: Vec<CrossBox> = vec![];
    let mut crossings: Vec<Crossing> = vec![];
    for line in b.lines() {
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        boxes.push(line.parse()?);
    }
    boxes.sort_unstable_by_key(|b| b.id);
    for line in c.lines() {
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        crossings.push(line.parse()?);
    }
    Ok((boxes, crossings))
}
