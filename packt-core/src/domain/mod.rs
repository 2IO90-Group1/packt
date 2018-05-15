pub mod problem;
pub mod solution;

pub use self::problem::Problem;
pub use self::solution::Solution;

use self::Rotation::*;
use failure::Error;
use rand::distributions::{IndependentSample, Normal};
use rand::{self, Rng};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    width: u32,
    height: u32,
    area: Option<u32>,
}

impl Rectangle {
    fn split(self, sp: Cut) -> (Rectangle, Rectangle) {
        let Rectangle {
            width: w,
            height: h,
            ..
        } = self;

        match sp {
            Cut::Horizontal(y) => {
                (Rectangle::new(w, h - y), Rectangle::new(w, y))
            }
            Cut::Vertical(x) => {
                (Rectangle::new(w - x, h), Rectangle::new(x, h))
            }
        }
    }

    fn gen_with_area(area: u32) -> Rectangle {
        let divisors: Vec<u32> =
            (1..=area).into_iter().filter(|i| area % i == 0).collect();

        let mut rng = rand::thread_rng();
        let n = divisors.len() as f64;
        let normal = Normal::new(n / 2., n / 7.);
        let i = normal.ind_sample(&mut rng).max(0.).min(n - 1.) as usize;

        let (width, height) = if rng.gen() {
            let width = divisors[i];
            (width, area / width)
        } else {
            let height = divisors[i];
            (area / height, height)
        };

        Rectangle {
            width,
            height,
            area: Some(area),
        }
    }

    fn simple_rsplit(self) -> (Rectangle, Rectangle) {
        let mut rng = rand::thread_rng();

        let cut = match (self.width, self.height) {
            (1, 1) => panic!("{:?} cannot be split", self),
            (1, h) if h > 1 => {
                let y = rng.gen_range(1, h);
                Cut::Horizontal(y)
            }
            (w, 1) if w > 1 => {
                let x = rng.gen_range(1, w);
                Cut::Vertical(x)
            }
            (w, h) if w > 1 && h > 1 => {
                if rng.gen_range(0, w + h) < w {
                    let x = rng.gen_range(1, w);
                    Cut::Vertical(x)
                } else {
                    let y = rng.gen_range(1, h);
                    Cut::Horizontal(y)
                }
            }
            _ => panic!("Unexpected input: {:?}", self),
        };

        self.split(cut)
    }

    fn area(&mut self) -> u32 {
        match self.area {
            Some(a) => a,
            None => {
                let a = self.width * self.height;
                self.area = Some(a);
                a
            }
        }
    }

    pub fn new(width: u32, height: u32) -> Rectangle {
        Rectangle {
            width,
            height,
            area: None,
        }
    }
}

enum Cut {
    Horizontal(u32),
    Vertical(u32),
}

impl fmt::Display for Rectangle {
    //noinspection RsTypeCheck
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{w} {h}", w = self.width, h = self.height)
    }
}

impl FromStr for Rectangle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let result = match s.split_whitespace()
            .collect::<Vec<&str>>()
            .as_slice()
        {
            [width, height] => Rectangle::new(width.parse()?, height.parse()?),
            _ => bail!("Invalid format: {}", s),
        };

        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotation {
    Normal,
    Rotated,
}

impl FromStr for Rotation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let result: Rotation = match s {
            "yes" => Rotation::Rotated,
            "no" => Rotation::Normal,
            _ => bail!("Unexpected token: {}", s),
        };

        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    rectangle: Rectangle,
    rotation: Rotation,
    bottom_left: Point,
    top_right: Point,
}

impl Placement {
    fn new(r: Rectangle, rotation: Rotation, bottom_left: Point) -> Placement {
        let (width, height) = match rotation {
            Normal => (r.width, r.height),
            Rotated => (r.height, r.width),
        };

        let x_max = bottom_left.x + width - 1;

        let y_max = bottom_left.y + height - 1;

        let top_right = Point::new(x_max, y_max);

        Placement {
            rectangle: r,
            rotation,
            bottom_left,
            top_right,
        }
    }

    fn overlaps(&self, rhs: &Placement) -> bool {
        rhs.bottom_left.y <= self.top_right.y
            && rhs.bottom_left.x <= self.top_right.x
            && self.bottom_left.y <= rhs.top_right.y
            && self.bottom_left.x <= rhs.top_right.x
    }
}
