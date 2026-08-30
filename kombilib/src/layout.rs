use crate::parsing;

pub(crate) fn compute_layout(boxes: &mut [parsing::CrossBox], crossings: &[parsing::Crossing]) {
    let mut placed = vec![false; boxes.len() + 1];
    let mut box_queue: Vec<usize> = vec![];
    box_queue.push(1);
    while let Some(box_id) = box_queue.pop() {
        placed[box_id] = true;
        println!(
            "Placing box {} at ({}, {})",
            box_id,
            boxes[box_id - 1].x,
            boxes[box_id - 1].y
        );
        crossings
            .iter()
            .filter(|c| c.id_a == box_id || c.id_b == box_id)
            .for_each(|c| {
                println!("Processing crossing {:?}", c);
                let other_box_id = if c.id_a == box_id { c.id_b } else { c.id_a };
                println!("Found crossing with box {}: {:?}", other_box_id, c);
                if !placed[other_box_id] {
                    println!(
                        "Placing box {} at ({}, {})",
                        other_box_id,
                        boxes[other_box_id - 1].x,
                        boxes[other_box_id - 1].y
                    );
                    println!(
                        "At index: {}, is box {:?}",
                        other_box_id - 1,
                        &boxes[other_box_id - 1]
                    );
                    let (x, y) = compute_box_position(
                        &boxes[box_id - 1],
                        &boxes[other_box_id - 1],
                        if c.id_a == box_id { c.c_a } else { c.c_b },
                        if c.id_a == box_id { c.c_b } else { c.c_a },
                    );
                    println!(
                        "Placing box {} at ({}, {}) based on crossing {:?}",
                        other_box_id, x, y, c
                    );
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
pub(crate) fn compute_box_position(
    box_a: &parsing::CrossBox,
    box_b: &parsing::CrossBox,
    c_a: i32,
    c_b: i32,
) -> (i32, i32) {
    let (x, y) = match (box_a.dir.clone(), box_b.dir.clone()) {
        (parsing::Direction::Down, parsing::Direction::Left) => {
            (box_a.x + c_b - 1, box_a.y + c_a - 1)
        }
        (parsing::Direction::Down, parsing::Direction::Right) => {
            (box_a.x - c_b + 1, box_a.y + c_a - 1)
        }
        (parsing::Direction::Up, parsing::Direction::Left) => {
            (box_a.x + c_b - 1, box_a.y - c_a + 1)
        }
        (parsing::Direction::Up, parsing::Direction::Right) => {
            (box_a.x - c_b + 1, box_a.y - c_a + 1)
        }
        (parsing::Direction::Left, parsing::Direction::Up) => {
            (box_a.x - c_a + 1, box_a.y + c_b - 1)
        }
        (parsing::Direction::Left, parsing::Direction::Down) => {
            (box_a.x - c_a + 1, box_a.y - c_b + 1)
        }
        (parsing::Direction::Right, parsing::Direction::Up) => {
            (box_a.x + c_a - 1, box_a.y + c_b - 1)
        }
        (parsing::Direction::Right, parsing::Direction::Down) => {
            (box_a.x + c_a - 1, box_a.y - c_b + 1)
        }
        _ => panic!(
            "Unsupported direction combination: {:?} and {:?}",
            box_a.dir, box_b.dir
        ),
    };
    (x, y)
}
