use clap::Parser;
use delaunator::{triangulate, Point, Triangulation};
use rand::{Rng, SeedableRng};
use svg::node::element::path::Data;
use svg::node::element::{Group, Path};
use svg::Document;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "800")]
    width: u32,
    #[arg(long, default_value = "600")]
    height: u32,
    #[arg(short = 'b', long, default_value = "#2980b9")]
    color_be: String,
    #[arg(short = 'e', long, default_value = "#ffffff")]
    color_ed: String,
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Debug, Clone)]
struct Triangle {
    a: Point,
    b: Point,
    c: Point,
}

impl Triangle {
    fn new(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> Self {
        Triangle {
            a: Point { x: a.0, y: a.1 },
            b: Point { x: b.0, y: b.1 },
            c: Point { x: c.0, y: c.1 },
        }
    }

    fn center(&self) -> Point {
        Point {
            x: (self.a.x + self.b.x + self.c.x) / 3.0,
            y: (self.a.y + self.b.y + self.c.y) / 3.0,
        }
    }

    fn generate_path_data(&self) -> Data {
        Data::new()
            .move_to((self.a.x, self.a.y))
            .line_to((self.b.x, self.b.y))
            .line_to((self.c.x, self.c.y))
            .close()
    }
}

#[derive(Debug, Clone, Copy)]
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

fn generate_points(width: u32, height: u32) -> Vec<Point> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    (0..100)
        .map(|_| {
            let x = rng.gen_range(0.0..width as f64);
            let y = rng.gen_range(0.0..height as f64);
            Point { x, y }
        })
        .collect()
}

fn extract_triangles(points: &[Point], delaunay: &Triangulation) -> Vec<Triangle> {
    (0..delaunay.len())
        .map(|i| {
            let x1 = points[delaunay.triangles[3 * i + 0]].x;
            let y1 = points[delaunay.triangles[3 * i + 0]].y;
            let x2 = points[delaunay.triangles[3 * i + 1]].x;
            let y2 = points[delaunay.triangles[3 * i + 1]].y;
            let x3 = points[delaunay.triangles[3 * i + 2]].x;
            let y3 = points[delaunay.triangles[3 * i + 2]].y;
            Triangle::new((x1, y1), (x2, y2), (x3, y3))
        })
        .collect()
}

fn generate_paths(triangles: &[Triangle], color_be: &str, color_ed: &str, width: u32) -> Vec<Path> {
    triangles
        .iter()
        .map(|triangle| {
            let loc = triangle.center().x + triangle.center().y / 2.0;
            let color = mix_colors(color_be, color_ed, loc, width);
            let data = triangle.generate_path_data();
            Path::new()
                .set("fill", color.clone())
                .set("stroke", color)
                .set("d", data)
        })
        .collect()
}

fn mix_colors(color_be: &str, color_ed: &str, loc: f64, width: u32) -> String {
    let start_color = Color::from_hex_code(color_be).unwrap();
    let end_color = Color::from_hex_code(color_ed).unwrap();
    let ratio = loc / width as f64;
    let r = (start_color.r as f64 * (1.0 - ratio) + end_color.r as f64 * ratio) as u8;
    let g = (start_color.g as f64 * (1.0 - ratio) + end_color.g as f64 * ratio) as u8;
    let b = (start_color.b as f64 * (1.0 - ratio) + end_color.b as f64 * ratio) as u8;
    Color::new(r, g, b).to_hex_code()
}

fn generate_document(paths: Vec<Path>, width: u32, height: u32) -> Document {
    let group = Group::new();
    let group = paths.into_iter().fold(group, |group, path| group.add(path));
    Document::new()
        .set("width", format!("{}", width))
        .set("height", format!("{}", height))
        .add(group)
}

fn output_result(
    file: &Option<String>,
    document: &Document,
) -> Result<(), Box<dyn std::error::Error>> {
    match file {
        Some(file) => svg::save(file, document)?,
        None => println!("{}", document),
    };
    Ok(())
}

fn main() {
    let args = Args::parse();

    let points = generate_points(args.width, args.height);
    let delaunay = triangulate(&points);
    let triangles = extract_triangles(&points, &delaunay);

    let paths = generate_paths(&triangles, &args.color_be, &args.color_ed, args.width);

    let document = generate_document(paths, args.width, args.height);

    output_result(&args.file, &document).unwrap();
}
