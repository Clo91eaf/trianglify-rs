use delaunator::{triangulate, Point};
use rand::{Rng, SeedableRng};
use svg::node::element::path::Data;
use svg::node::element::{Group, Path};
use svg::Document;

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[allow(dead_code)]
impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
    fn from_hex_code(hex_code: &str) -> Option<Self> {
        if hex_code.len() != 7 || !hex_code.starts_with("#") {
            return None;
        }

        let r = u8::from_str_radix(&hex_code[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex_code[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex_code[5..7], 16).ok()?;

        Some(Color::new(r, g, b))
    }
    fn to_hex_code(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn main() {
    let mut rng = rand::rngs::StdRng::from_entropy();

    let (width, height) = (800, 600);

    let (color_be, color_ed) = (
        Color::from_hex_code("#b92b27").unwrap(),
        Color::from_hex_code("#1565c0").unwrap(),
    );

    let points: Vec<Point> = (0..100)
        .map(|_| {
            let x = rng.gen_range(0.0..width as f64);
            let y = rng.gen_range(0.0..height as f64);
            Point { x, y }
        })
        .collect();

    let delaunay = triangulate(&points);

    let triangles: Vec<((f64, f64), (f64, f64), (f64, f64))> = (0..delaunay.len())
        .enumerate()
        .map(|(i, _)| {
            let x1 = points[delaunay.triangles[3 * i + 0]].x;
            let y1 = points[delaunay.triangles[3 * i + 0]].y;
            let x2 = points[delaunay.triangles[3 * i + 1]].x;
            let y2 = points[delaunay.triangles[3 * i + 1]].y;
            let x3 = points[delaunay.triangles[3 * i + 2]].x;
            let y3 = points[delaunay.triangles[3 * i + 2]].y;
            ((x1, y1), (x2, y2), (x3, y3))
        })
        .collect();

    let paths: Vec<Path> = triangles
        .into_iter()
        .map(|((x1, y1), (x2, y2), (x3, y3))| {
            let data = Data::new()
                .move_to((x1, y1))
                .line_to((x2, y2))
                .line_to((x3, y3))
                .close();
            let loc = (x1 + x2 + x3 + y1 + y2 + y3) / 6.0;
            let color = Color {
                r: ((color_be.r as f64) * (loc / width as f64)
                    + (color_ed.r as f64) * (1.0 - loc / width as f64)) as u8,
                g: ((color_be.g as f64) * (loc / width as f64)
                    + (color_ed.g as f64) * (1.0 - loc / width as f64)) as u8,
                b: ((color_be.b as f64) * (loc / width as f64)
                    + (color_ed.b as f64) * (1.0 - loc / width as f64)) as u8,
            };
            let color = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);
            Path::new()
                .set("fill", color)
                .set("stroke", "none")
                .set("d", data)
        })
        .collect();

    let group = Group::new();
    let group = paths.into_iter().fold(group, |group, path| group.add(path));

    let document = Document::new()
        .set("width", format!("{}", width))
        .set("height", format!("{}", height))
        .add(group);

    println!("{}", document);
}
