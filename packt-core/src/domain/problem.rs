use domain::Rectangle;
use failure::Error;
use rand::{self, seq, Rng};
use std::cmp::min;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

const N_DEFAULTS: [usize; 5] = [3, 5, 10, 25, 10000];

const AVG_RECTANGLE_AREA: usize = 100;

#[derive(Clone, Debug, PartialEq)]

pub struct Problem {
    pub variant: Variant,
    pub allow_rotation: bool,
    pub rectangles: Vec<Rectangle>,
}

impl Problem {
    pub fn generator() -> ProblemGenerator {
        ProblemGenerator::default()
    }

    // TODO: Add rotated rectangles
    fn generate_from(
        r: Rectangle,
        n: usize,
        v: Variant,
        allow_rotation: bool,
    ) -> Problem {
        let a = r.width * r.height;

        if n > a {
            panic!("{:?} cannot be split into {} rectangles", r, n)
        } else if n == a {
            let rectangles = vec![Rectangle::new(1, 1); n];

            return Problem {
                variant: v,
                allow_rotation,
                rectangles,
            };
        }

        let mut rng = rand::thread_rng();

        let mut rectangles = Vec::with_capacity(n);

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
        }
    }

    fn config(&self) -> String {
        format!(
            "container height: {v}\nrotations allowed: {r}\nnumber of \
             rectangles: {n}\n",
            v = self.variant,
            r = if self.allow_rotation {
                "yes"
            } else {
                "no"
            },
            n = self.rectangles.len()
        )
    }

    pub fn digest(&self) -> String {
        let mut s = self.config();

        self.rectangles
            .iter()
            .take(30)
            .for_each(|r| s.push_str(&format!("\n{}", r.to_string())));

        if self.rectangles.len() > 30 {
            s.push_str("\n...");
        }

        s
    }
}

impl fmt::Display for Problem {
    //noinspection RsTypeCheck
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = self.config();

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
            .ok_or_else(|| format_err!("Unexpected end of file"))?
            .split_whitespace()
            .collect();

        let variant = match l1.as_slice() {
            ["container", "height:", "free"] => Variant::Free,
            ["container", "height:", "fixed", h] => Variant::Fixed(h.parse()?),
            _ => bail!("Invalid format: {}", l1.join(" ")),
        };

        let l2 = lines
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file"))?;

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
        })
    }
}

#[derive(Default)]

pub struct ProblemGenerator {
    container: Option<Rectangle>,
    rectangles: Option<usize>,
    variant: Option<Variant>,
    allow_rotation: Option<bool>,
}

impl ProblemGenerator {
    fn new() -> Self {
        Self::default()
    }

    pub fn generate(&self) -> Problem {
        let mut rng = rand::thread_rng();

        let mut n = self.rectangles
            .unwrap_or_else(|| seq::sample_slice(&mut rng, &N_DEFAULTS, 1)[0]);

        let mut r = self.container.unwrap_or_else(|| {
            let area = n * AVG_RECTANGLE_AREA;

            Rectangle::gen_with_area(area)
        });

        n = min(n, r.area());

        let variant = self.variant.unwrap_or_else(|| {
            if rng.gen() {
                Variant::Free
            } else {
                Variant::Fixed(r.height)
            }
        });

        let allow_rotation = self.allow_rotation.unwrap_or_else(|| rng.gen());

        Problem::generate_from(r, n, variant, allow_rotation)
    }

    pub fn rectangles(mut self, mut n: usize) -> Self {
        if let Some(ref mut r) = self.container {
            n = min(n, r.area());
        }

        self.rectangles = Some(n);

        self
    }

    pub fn allow_rotation(mut self, b: bool) -> Self {
        self.allow_rotation = Some(b);

        self
    }

    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = Some(v);

        self
    }

    pub fn container(mut self, mut r: Rectangle) -> Self {
        self.container = Some(r);

        self.rectangles.map(|n| min(n, r.area()));

        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]

pub enum Variant {
    Free,
    Fixed(usize),
}

impl fmt::Display for Variant {
    //noinspection RsTypeCheck
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

    const input: &str = "container height: fixed 22\nrotations allowed: \
                         no\nnumber of rectangles: 2\n12 8\n10 9";

    #[test]
    fn parsing() {
        let expected = Problem {
            variant: Variant::Fixed(22),
            allow_rotation: false,
            rectangles: vec![Rectangle::new(12, 8), Rectangle::new(10, 9)],
        };

        let result: Problem = input.parse().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn format_parse() {
        assert_eq!(
            input,
            format!("{}", input.parse::<Problem>().unwrap())
        )
    }

    #[test]
    fn generate_from() {
        let r = Rectangle::new(1000, 1000);
        let p = Problem::generate_from(r, 50, Variant::Free, false);
        let a: usize = p.rectangles
            .into_iter()
            .map(|r| r.height * r.width)
            .sum();

        assert_eq!(a, 1000 * 1000);
    }

}