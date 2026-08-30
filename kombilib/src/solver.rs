use crate::parsing;

use crate::output::Square;

use super::Instance;

pub(crate) fn solve(instance: &Instance) -> Vec<Square> {
    generate_squares(&instance.boxes)
}

pub(crate) fn generate_squares(boxes: &[parsing::CrossBox]) -> Vec<Square> {
    boxes
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
        .collect()
}
