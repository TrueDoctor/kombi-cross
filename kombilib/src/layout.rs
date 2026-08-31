use crate::parsing;

pub(crate) fn compute_layout(boxes: &mut [parsing::CrossBox], crossings: &[parsing::Crossing]) {
    let mut placed = vec![false; boxes.len()];
    let mut box_queue: Vec<usize> = vec![];
    box_queue.push(0);
    while let Some(box_id) = box_queue.pop() {
        placed[box_id] = true;
        crossings
            .iter()
            .filter(|c| c.id_a == box_id || c.id_b == box_id)
            .for_each(|c| {
                let other_box_id = if c.id_a == box_id { c.id_b } else { c.id_a };
                if !placed[other_box_id] {
                    let (x, y) = compute_box_position(
                        &boxes[box_id],
                        &boxes[other_box_id],
                        if c.id_a == box_id { c.c_a } else { c.c_b },
                        if c.id_a == box_id { c.c_b } else { c.c_a },
                    );
                    boxes[other_box_id].x = x;
                    boxes[other_box_id].y = y;
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

pub(crate) fn compute_box_position(
    box_a: &parsing::CrossBox,
    box_b: &parsing::CrossBox,
    c_a: i32,
    c_b: i32,
) -> (i32, i32) {
    use parsing::Direction::*;
    match (box_a.dir.clone(), box_b.dir.clone()) {
        (Down, Left) => (box_a.x + c_b, box_a.y + c_a),
        (Down, Right) => (box_a.x - c_b, box_a.y + c_a),
        (Up, Left) => (box_a.x + c_b, box_a.y - c_a),
        (Up, Right) => (box_a.x - c_b, box_a.y - c_a),
        (Left, Up) => (box_a.x - c_a, box_a.y + c_b),
        (Left, Down) => (box_a.x - c_a, box_a.y - c_b),
        (Right, Up) => (box_a.x + c_a, box_a.y + c_b),
        (Right, Down) => (box_a.x + c_a, box_a.y - c_b),
        _ => panic!(
            "Unsupported direction combination: {:?} and {:?}",
            box_a.dir, box_b.dir
        ),
    }
}
