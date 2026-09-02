use std::error::Error;

use crate::parsing::{CrossBox, Crossing};

mod layout;
mod output;
mod parsing;
mod solver;

fn main() -> Result<(), Box<dyn Error>> {
    let input = std::fs::read_to_string("input_2023").unwrap();
    let (b, c, s) = match input.split("---").collect::<Vec<_>>().as_slice() {
        [b, c, s] => (*b, *c, *s),
        _ => panic!("input must contain exactly three sections separated by ---"),
    };
    let words = std::fs::read_to_string("words_2023").unwrap();
    let words: Vec<_> = words.lines().map(parsing::umlaut).filter(|w| !w.is_empty() && w != " ").collect();

    let (mut boxes, crossings, solution_cells) = parsing::parse_file(b, c, s)?;

    layout::compute_layout(&mut boxes, &crossings);

    let instance = Instance {
        boxes,
        crossings,
        words,
        solution_cells,
    };
    let solver = solver::Solver::new(dbg!(instance));
    let solutions = solver.solve();
    println!("found {} solutions", solutions.len());
    let consensus = solver.consensus(&solutions);
    std::fs::write(
        "out.svg",
        output::format_svg(solver.format_state(&consensus)),
    )?;
    solver.print_word_status(&consensus);
    solver.print_solution_status(&consensus);
    Ok(())
}

#[derive(Clone, Debug)]
struct Instance {
    boxes: Vec<CrossBox>,
    crossings: Vec<Crossing>,
    words: Vec<String>,
    solution_cells: Vec<(u32, u32)>,
}
