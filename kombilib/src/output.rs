const WIDTH: i32 = 25;
const MARGIN: i32 = 5;

pub struct Square {
    pub x: i32,
    pub y: i32,
    pub char: String,
}

impl Square {
    fn format(&self) -> String {
        format!("+
                <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{w}\" stroke=\"black\" fill=\"none\" stroke-width=\"5\"/>
                <text x=\"{x}\" y=\"{y}\">{c}</text>",
            x = self.x  * WIDTH + MARGIN,
            y = self.y * WIDTH + MARGIN,
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
        width * WIDTH + 2 * MARGIN,
        height * WIDTH + 2 * MARGIN
    );

    for square in boxes {
        svg.push_str(&square.format());
    }

    svg.push_str("</svg>");
    svg
}
