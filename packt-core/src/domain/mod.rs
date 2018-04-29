pub mod problem;
pub mod solution;

pub use self::problem::Problem;
pub use self::solution::Solution;

use self::Rotation::*;
use failure::Error;
use rand::{thread_rng, Rng};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    width: usize,
    height: usize,
    area: Option<usize>,
}

impl Rectangle {
    fn new(width: usize, height: usize) -> Rectangle {
        Rectangle {
            width,
            height,
            area: None,
        }
    }

    fn area(&mut self) -> usize {
        match self.area {
            Some(a) => a,
            None => {
                let a = self.width * self.height;
                self.area = Some(a);
                a
            }
        }
    }

    /// Splits this rectangle.
    ///
    /// # Panics
    ///
    /// This function will panic if `self.width <= 1 && self.height <= 1`.
    fn simple_rsplit(self) -> (Self, Self) {
        let mut rng = thread_rng();
        let method = match (self.width, self.height) {
            (1, 1) => panic!("{:?} cannot be split", self),
            (1, h) if h > 1 => {
                let y = rng.gen_range(1, h);
                Split::Horizontal(y)
            }
            (w, 1) if w > 1 => {
                let x = rng.gen_range(1, w);
                Split::Vertical(x)
            }
            (w, h) if w > 1 && h > 1 => {
                let x = rng.gen_range(1, w);
                let y = rng.gen_range(1, h);

                if rng.gen() {
                    Split::Vertical(x)
                } else {
                    Split::Horizontal(y)
                }
            }
            _ => panic!("Unexpected input: {:?}", self),
        };

        self.split(method)
    }

    fn gen_with_area(area: usize) -> Self {
        let width = thread_rng().gen_range(1, area + 1);

        Rectangle {
            width,
            height: area / width,
            area: Some(area),
        }
    }

    fn split(self, sp: Split) -> (Self, Self) {
        let Rectangle {
            width: w,
            height: h,
            ..
        } = self;

        match sp {
            Split::Horizontal(y) => {
                (Rectangle::new(w, h - y), Rectangle::new(w, y))
            }
            Split::Vertical(x) => {
                (Rectangle::new(w - x, h), Rectangle::new(x, h))
            }
        }
    }
}

enum Split {
    Horizontal(usize),
    Vertical(usize),
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
        let result = match s {
            "yes" => Rotated,
            "no" => Normal,
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

        let x_max = bottom_left.x + width;
        let y_max = bottom_left.y + height;
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
