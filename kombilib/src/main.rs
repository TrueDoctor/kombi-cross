use std::error::Error;

use crate::output::Square;

mod layout;
mod output;
mod parsing;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let input = std::fs::read_to_string("input").unwrap();
    let (b, c) = input.split_once("---").unwrap();
    let words = std::fs::read_to_string("words").unwrap();
    let _words: Vec<_> = words.lines().map(parsing::umlaut).collect();

    let mut boxes: Vec<parsing::CrossBox> = vec![];
    let mut crossings: Vec<parsing::Crossing> = vec![];

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

    layout::compute_layout(&mut boxes, &crossings);

    for b in &boxes {
        println!("{:?}", b);
    }

    let squares = boxes
        .iter()
        .flat_map(|b| {
            (0..b.len).map(move |i| {
                let i = i as i32;
                let (x, y) = match b.dir {
                    parsing::Direction::Left => (b.x - i, b.y),
                    parsing::Direction::Right => (b.x + i, b.y),
                    parsing::Direction::Up => (b.x, b.y - i),
                    parsing::Direction::Down => (b.x, b.y + i),
                };
                Square {
                    x,
                    y,
                    char: String::new(),
                }
            })
        })
        .collect();
    std::fs::write("out.svg", output::format_svg(squares))?;
    Ok(())
}
