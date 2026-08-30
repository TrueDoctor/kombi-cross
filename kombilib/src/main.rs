use std::{error::Error, str::FromStr};

mod output;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let input = std::fs::read_to_string("input").unwrap();
    let (b, c) = input.split_once("---").unwrap();

    let mut boxes: Vec<CrossBox> = vec![];
    let mut crossings: Vec<Crossing> = vec![];

    for line in b.lines() {
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        boxes.push(line.parse()?);
    }
    for line in c.lines() {
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        crossings.push(line.parse()?);
    }

    dbg!(&crossings);

    let squares = vec![];
    std::fs::write("out.svg", output::format_svg(squares))?;
    Ok(())
}

#[derive(Clone, Debug)]
struct CrossBox {
    id: usize,
    len: usize,
    dir: Direction,
}

impl FromStr for CrossBox {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_ascii_whitespace();
        let id = split.next().unwrap().parse().unwrap();
        let len = split.next().unwrap().parse().unwrap();
        let dir = split.next().unwrap().parse().unwrap();

        Ok(CrossBox { id, len, dir })
    }
}

#[derive(Clone, Debug)]
enum Direction {
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
struct Crossing {
    id_a: usize,
    id_b: usize,
    c_a: usize,
    c_b: usize,
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
