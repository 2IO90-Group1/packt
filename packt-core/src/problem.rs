use failure::Error;
use geometry::Rectangle;
use rand::{self, seq, Rng};
use std::cmp::min;
use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::FromStr;

const N_DEFAULTS: [usize; 5] = [3, 5, 10, 25, 5000];
const AVG_RECTANGLE_AREA: u64 = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub variant: Variant,
    pub allow_rotation: bool,
    pub rectangles: Vec<Rectangle>,
    pub source: Option<Rectangle>,
}

impl Problem {
    // TODO: Add rotated rectangles
    fn generate_from(r: Rectangle, n: usize, v: Variant, allow_rotation: bool) -> Problem {
        let a = r.area() as usize;
        if n > a {
            panic!("{:?} cannot be split into {} rectangles", r, n)
        } else if n == a {
            let rectangles = vec![Rectangle::new(1, 1); n];
            return Problem {
                variant: v,
                allow_rotation,
                rectangles,
                source: None,
            };
        }

        let mut rng = rand::thread_rng();
        let mut rectangles = Vec::with_capacity(n as usize);
        rectangles.push(r);

        while rectangles.len() < n {
            let i = seq::sample_indices(&mut rng, rectangles.len(), 1)[0];
            let r = rectangles.swap_remove(i);

            if r.width > 1 || r.height > 1 {
                let (r1, r2) = r.simple_rsplit();
                rectangles.push(r1);
                rectangles.push(r2);
            } else {
                rectangles.push(r);
            }
        }

        Problem {
            variant: v,
            allow_rotation,
            rectangles,
            source: Some(r),
        }
    }

    fn config_str(&self) -> String {
        format!(
            "container height: {v}\nrotations allowed: {r}\nnumber of rectangles: {n}",
            v = self.variant,
            r = if self.allow_rotation { "yes" } else { "no" },
            n = self.rectangles.len()
        )
    }

    pub fn digest(&self) -> String {
        let mut config = self.config_str();

        if let Some(source) = self.source {
            config.push_str(&format!("\nbounding box: {}", source.to_string()));
        }

        self.rectangles
            .iter()
            .for_each(|r| config.push_str(&format!("\n{}", r.to_string())));

        config
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).open(path)?;

        file.write_all(self.to_string().as_bytes())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Problem, Error> {
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        content.parse()
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = self.config_str();

        self.rectangles
            .iter()
            .for_each(|r| s.push_str(&format!("\n{}", r.to_string())));

        write!(f, "{}", s)
    }
}

impl FromStr for Problem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut lines = s.lines();
        let l1: Vec<&str> = lines
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file: unable to parse problem variant"))?
            .split_whitespace()
            .collect();

        let variant = match l1.as_slice() {
            ["container", "height:", "free"] => Variant::Free,
            ["container", "height:", "fixed", h] => Variant::Fixed(h.parse()?),
            _ => bail!("Invalid format: {}", l1.join(" ")),
        };

        let l2 = lines.next().ok_or_else(|| {
            format_err!("Unexpected end of file: unable to parse problem rotation setting")
        })?;

        let allow_rotation = match l2 {
            "rotations allowed: yes" => true,
            "rotations allowed: no" => false,
            _ => bail!("Invalid format: {}", l2),
        };

        lines.next();
        let rectangles = lines
            .map(|s| s.parse())
            .collect::<Result<Vec<Rectangle>, _>>()?;

        Ok(Problem {
            variant,
            allow_rotation,
            rectangles,
            source: None,
        })
    }
}

#[derive(Default)]
pub struct Generator {
    container: Option<Rectangle>,
    rectangles: Option<usize>,
    variant: Option<Variant>,
    allow_rotation: Option<bool>,
}

impl Generator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(&self) -> Problem {
        let mut rng = rand::thread_rng();
        let mut n = self
            .rectangles
            .unwrap_or_else(|| seq::sample_slice(&mut rng, &N_DEFAULTS, 1)[0]);

        let r = self.container.unwrap_or_else(|| {
            let area = n as u64 * AVG_RECTANGLE_AREA;

            Rectangle::gen_with_area(area)
        });

        n = min(n, r.area() as usize);
        let variant = self
            .variant
            .map(|v| match v {
                Variant::Fixed(_h) => Variant::Fixed(r.height),
                v => v,
            })
            .unwrap_or_else(|| {
                if rng.gen() {
                    Variant::Free
                } else {
                    Variant::Fixed(r.height)
                }
            });

        let allow_rotation = self.allow_rotation.unwrap_or_else(|| rng.gen());
        Problem::generate_from(r, n, variant, allow_rotation)
    }

    pub fn rectangles(&mut self, mut n: usize) {
        if let Some(ref mut r) = self.container {
            n = min(n, r.area() as usize);
        }

        self.rectangles = Some(n);
    }

    pub fn allow_rotation(&mut self, b: bool) {
        self.allow_rotation = Some(b);
    }

    pub fn variant(&mut self, v: Variant) {
        self.variant = Some(v);
    }

    pub fn container(&mut self, r: Rectangle) {
        self.container = Some(r);
        self.rectangles.map(|n| min(n, r.area() as usize));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Variant {
    Free,
    Fixed(u32),
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Variant::Free => write!(f, "free"),
            Variant::Fixed(h) => write!(f, "fixed {}", h),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_upper_case_globals)]
    use super::*;
    const input: &str =
        "container height: fixed 22\nrotations allowed: no\nnumber of rectangles: 2\n12 8\n10 9";

    #[test]
    fn parsing() {
        let expected = Problem {
            variant: Variant::Fixed(22),
            allow_rotation: false,
            rectangles: vec![Rectangle::new(12, 8), Rectangle::new(10, 9)],
            source: None,
        };

        let result: Problem = input.parse().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn format_parse() {
        assert_eq!(input, format!("{}", input.parse::<Problem>().unwrap()))
    }

    #[test]
    fn generate_from() {
        let r = Rectangle::new(1000, 1000);
        let p = Problem::generate_from(r, 50, Variant::Free, false);
        let a: u32 = p.rectangles.into_iter().map(|r| r.height * r.width).sum();

        assert_eq!(a, 1000 * 1000);
    }

}
