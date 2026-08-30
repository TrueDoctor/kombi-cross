use std::{error::Error, str::FromStr};

use crate::output::Square;

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

    compute_layout(&mut boxes, &crossings);

    for b in &boxes {
        println!("{:?}", b);
    }

    let squares = boxes.iter().flat_map(|b| (0..b.len).map(move |i| {
        let i = i as i32;
        let (x, y) = match b.dir {
            Direction::Left => (b.x - i, b.y),
            Direction::Right => (b.x + i, b.y),
            Direction::Up => (b.x, b.y - i),
            Direction::Down => (b.x, b.y + i),
        };
        Square {
            x: x,
            y: y,
            char: format!(""),
        }
    })).collect();
    std::fs::write("out.svg", output::format_svg(squares))?;
    Ok(())
}

#[derive(Clone, Debug)]
struct CrossBox {
    x: i32,
    y: i32,
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

        Ok(CrossBox { x: 0, y: 0, id, len, dir })
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
    c_a: i32,
    c_b: i32,
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

fn compute_layout(boxes: &mut Vec<CrossBox>, crossings: &[Crossing]) {
    let mut placed = vec![false; boxes.len() + 1];
    let mut box_queue: Vec<usize> = vec![];
    box_queue.push(1);
    while !box_queue.is_empty() {
        let box_id = box_queue.pop().unwrap();
        placed[box_id] = true;
        println!("Placing box {} at ({}, {})", box_id, boxes[box_id - 1].x, boxes[box_id - 1].y);
        crossings.iter()
            .filter(|c| c.id_a == box_id || c.id_b == box_id)
            .for_each(|c| {
                println!("Processing crossing {:?}", c);
                let other_box_id = if c.id_a == box_id { c.id_b } else { c.id_a };
                println!("Found crossing with box {}: {:?}", other_box_id, c);
                if !placed[other_box_id] {
                    println!("Placing box {} at ({}, {})", other_box_id, boxes[other_box_id - 1].x, boxes[other_box_id - 1].y);
                    println!("At index: {}, is box {:?}", other_box_id - 1, &boxes[other_box_id - 1]);
                    let (x, y) = compute_box_position(&boxes[box_id - 1], &boxes[other_box_id - 1], if c.id_a == box_id { c.c_a } else { c.c_b }, if c.id_a == box_id { c.c_b } else { c.c_a });
                    println!("Placing box {} at ({}, {}) based on crossing {:?}", other_box_id, x, y, c);
                    boxes[other_box_id - 1].x = x;
                    boxes[other_box_id - 1].y = y;
                    placed[other_box_id] = true;
                    box_queue.push(other_box_id);
                }
            });
    }
    let min_y = boxes.iter().map(|b| b.y).min().unwrap_or_default();
    if min_y < 0 {
        for b in boxes.iter_mut() {
            b.y -= min_y;
        }
    }
    let min_x = boxes.iter().map(|b| b.x).min().unwrap_or_default();
    if min_x < 0 {
        for b in boxes.iter_mut() {
            b.x -= min_x;
        }
    }
}

// 0,0 is the top left corner
// the c_a and c_b values denote that the crossing is at the c_th square of the box in the direction
fn compute_box_position(box_a: &CrossBox, box_b: &CrossBox, c_a: i32, c_b: i32) -> (i32, i32) {
    let (x, y) = match (box_a.dir.clone(), box_b.dir.clone()) {
        (Direction::Down, Direction::Left) => (box_a.x + c_b - 1, box_a.y + c_a - 1),
        (Direction::Down, Direction::Right) => (box_a.x - c_b + 1, box_a.y + c_a - 1),
        (Direction::Up, Direction::Left) => (box_a.x + c_b - 1, box_a.y - c_a + 1),
        (Direction::Up, Direction::Right) => (box_a.x - c_b + 1, box_a.y - c_a + 1),
        (Direction::Left, Direction::Up) => (box_a.x - c_a + 1, box_a.y + c_b - 1),
        (Direction::Left, Direction::Down) => (box_a.x - c_a + 1, box_a.y - c_b + 1),
        (Direction::Right, Direction::Up) => (box_a.x + c_a - 1, box_a.y + c_b - 1),
        (Direction::Right, Direction::Down) => (box_a.x + c_a - 1, box_a.y - c_b + 1),
        _ => panic!("Unsupported direction combination: {:?} and {:?}", box_a.dir, box_b.dir),
    };
    (x, y)
}
