#[derive(Default)]
pub struct TextStyle {
    bold: bool,
    italic: bool,
    underlined: bool,
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct Screen {
    width: u32,
    height: u32,
}

impl Screen {
    pub fn init() -> Screen {
        todo!();
    }

    // pub fn write_text(&self, line: u32, msg: String, style: TextStyle) {
    //     todo!();
    // }

    pub fn draw_pixel(&self, x: u32, y: u32, color: Color) {
        todo!();
    }

    pub fn draw_line(&self, start: (u32, u32), end: (u32, u32), color: Color) {
        todo!();
    }

    pub fn draw_rect(&self, start: (u32, u32), end: (u32, u32), color: Color, fill: bool) {
        todo!();
    }

    pub fn draw_ellipse(&self, center: (u32, u32), size: u32, color: Color, fill: bool) {
        todo!();
    }

    // pub fn draw_polygon(&self, points: Vec<u32>, color: Color) {
    //     assert!(points.len() < 2, "Not enough points to draw the polygon");
    //     todo!();
    // }

    pub fn fill_screen(&self, color: Color) {
        todo!();
    }
}
