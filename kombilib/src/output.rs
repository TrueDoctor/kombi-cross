const WIDTH: i32 = 25;
const MARGIN: i32 = 5;

pub struct Square {
    pub x: i32,
    pub y: i32,
    pub char: String,
}

impl Square {
    fn format(&self) -> String {
        format!("
                <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{w}\" stroke=\"black\" fill=\"#fcfcfc\" stroke-width=\"5\"/>
                <text x=\"{cx}\" y=\"{cy}\" font-family=\"sans-serif\" font-weight=\"bold\" font-size=\"{fs}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{c}</text>",
            x = self.x  * WIDTH + MARGIN,
            y = self.y * WIDTH + MARGIN,
            cx = self.x * WIDTH + MARGIN + WIDTH / 2,
            cy = self.y * WIDTH + MARGIN + WIDTH * 3 / 5,
            w = WIDTH,
            fs = WIDTH * 3 / 5,
            c = self.char
        )
    }
}

pub fn format_svg(boxes: Vec<Square>) -> String {
    let mut boxes = boxes;
    // render empty boxes first so that they are below the filled boxes
    boxes.sort_by(|a, b| a.char.cmp(&b.char));
    let width = boxes.iter().map(|s| s.x).max().unwrap_or_default() + 1;
    let height = boxes.iter().map(|s| s.y).max().unwrap_or_default() + 1;

    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        width * WIDTH + 2 * MARGIN,
        height * WIDTH + 2 * MARGIN
    );

    let background_fill = format!(
        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#9dd3e3\"/>",
        width * WIDTH + 2 * MARGIN,
        height * WIDTH + 2 * MARGIN
    );

    svg.push_str(&background_fill);

    for square in boxes {
        svg.push_str(&square.format());
    }

    svg.push_str("</svg>");
    svg
}
