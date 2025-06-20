use std::path::Path;

use argp::FromArgs;
use image::{DynamicImage, GenericImageView};

struct Circle {
    pub centre_x: f64,
    pub centre_y: f64,
    pub radius: f64,
    pub colour: String,
}

impl Circle {
    pub fn overlaps(&self, other: &Circle, spacing: f64) -> bool {
        (self.centre_x - other.centre_x).powi(2) + (self.centre_y - other.centre_y).powi(2)
            < (self.radius + other.radius + spacing).powi(2)
    }

    pub fn new(radius: f64, width: f64) -> Circle {
        loop {
            let x = radius + fastrand::f64() * (width - 2. * radius);
            let y = radius + fastrand::f64() * (width - 2. * radius);

            if (x - width * 0.5).powi(2) + (y - width * 0.5).powi(2)
                < (width * 0.5 - radius).powi(2)
            {
                return Circle {
                    centre_x: x,
                    centre_y: y,
                    radius,
                    colour: "black".to_string()
                };
            }
        }
    }

    pub fn svg(&self) -> String {
        format!(
            r#"<circle r="{}" cx="{}" cy="{}" fill="{}" />"#,
            self.radius, self.centre_x, self.centre_y, self.colour
        )
    }
}

/// Main command
#[derive(FromArgs)]
struct Command {
    /// Dimensions of the image
    #[argp(option, short = 'w')]
    width: Option<f64>,

    /// Passes as a comma separataed list of decimal and whole numbers,
    /// odd numbers digits represent radius, even represent number of passes.
    #[argp(option, short = 'p')]
    passes: Option<String>,

    /// Minimum separation between circles
    #[argp(option, short = 's')]
    spacing: Option<f64>,

    /// Image of the source file
    #[argp(option, short = 'i')]
    img: Option<String>,
}

impl Command {
    pub fn run(&self) {
        let width = self.width.unwrap_or(1000.);
        let passes: Vec<(f64, u32)> = match &self.passes {
            Some(v) => v
                .split(',')
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|chunks| (chunks[0].parse().unwrap(), chunks[1].parse().unwrap()))
                .collect(),
            None => vec![
                (width * 0.035, 20),
                (width * 0.032, 850),
                (width * 0.030, 700),
                (width * 0.025, 1500),
                (width * 0.021, 5000),
                (width * 0.016, 10000),
                (width * 0.013, 1000000),
                (width * 0.08, 1000000),
            ],
        };

        let spacing = self.spacing.unwrap_or(width * 0.0004);

        let mut circles = Vec::new();

        for (radius, samples) in passes {
            for _ in 0..samples {
                let circle = Circle::new(radius, width);

                if circles.iter().all(|other| !circle.overlaps(other, spacing)) {
                    circles.push(circle);
                }
            }
        }

        if let Some(img) = &self.img {
            fn get_colour(circle: &Circle, width: f64, image: &DynamicImage) -> String {
                let img_width = image.dimensions().0;
                let img_height = image.dimensions().1;

                let x = (img_width as f64 / width * circle.centre_x).floor() as u32;
                let y = (img_height as f64 / width * circle.centre_y).floor() as u32;

                let pixel = image.get_pixel(x, y);
                format!("rgba({})", pixel.0.map(|n| n.to_string()).join(","))
            }

            let image = image::open(&Path::new(img)).unwrap();

            for circle in circles.iter_mut() {
                circle.colour = get_colour(circle, width, &image)
            }
        }

        let svg = circles
            .iter()
            .map(Circle::svg)
            .fold(String::new(), |acc, current| acc + "  " + &current + "\n");

        println!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {width}">
{svg}</svg>"#
        )
    }
}

fn main() {
    let cmd: Command = argp::parse_args_or_exit(argp::DEFAULT);
    cmd.run();
}
