use std::error::Error;

use crate::parsing::{CrossBox, Crossing};

mod layout;
mod output;
mod parsing;
mod solver;

fn main() -> Result<(), Box<dyn Error>> {
    let input = std::fs::read_to_string("input").unwrap();
    let (b, c) = input.split_once("---").unwrap();
    let words = std::fs::read_to_string("words").unwrap();
    let words: Vec<_> = words.lines().map(parsing::umlaut).collect();

    let (mut boxes, crossings) = parsing::parse_file(b, c)?;

    layout::compute_layout(&mut boxes, &crossings);

    let instance = Instance {
        boxes,
        crossings,
        words,
    };
    let solver = solver::Solver::new(instance);
    let solutions = solver.solve();
    println!("found {} solutions", solutions.len());
    let consensus = solver.consensus(&solutions);
    std::fs::write(
        "out.svg",
        output::format_svg(solver.format_state(&consensus)),
    )?;
    Ok(())
}

struct Instance {
    boxes: Vec<CrossBox>,
    crossings: Vec<Crossing>,
    words: Vec<String>,
}
