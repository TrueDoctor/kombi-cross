const WIDTH: usize = 1;

pub struct Square {
    pub x: usize,
    pub y: usize,
    pub char: String,
}

impl Square {
    fn format(&self) -> String {
        format!("
            <g>
                <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{w}\" stroke=\"black\" fill=\"none\" stroke-width=\"5\"/>
                <text x=\"{x}\" y=\"{y}\">{c}</text>
            </g>",
            x = self.x  * WIDTH,
            y = self.y * WIDTH,
            w = WIDTH,
            c = self.char
        )
    }
}

pub fn format_svg(boxes: Vec<Square>) -> String {
    let width = boxes.iter().map(|s| s.x).max().unwrap_or_default() + 1;
    let height = boxes.iter().map(|s| s.y).max().unwrap_or_default() + 1;

    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        width * WIDTH,
        height * WIDTH
    );

    for square in boxes {
        svg.push_str(&square.format());
    }

    svg.push_str("</svg>");
    svg
}
