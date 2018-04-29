use std::str::FromStr;
use failure::Error;
use rand::{Rng, thread_rng};

pub mod problem;
pub mod solution;

pub use self::problem::Problem;
pub use self::solution::Solution;

use self::Rotation::*;

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
}

impl Rectangle {
    fn new(width: usize, height: usize) -> Rectangle {
        Rectangle { width, height }
    }

    /// Splits this rectangle.
    ///
    /// # Panics
    ///
    /// This function will panic if `self.width <= 1 && self.height <= 1`.
    fn simple_rsplit(self) -> (Self, Self) {
        if thread_rng().gen() && self.height > 1 {
            let y = thread_rng().gen_range(1, self.height);
            self.split(Split::Horizontal(y))
        } else if self.width > 1 {
            let x = thread_rng().gen_range(1, self.width);
            self.split(Split::Vertical(x))
        } else {
            panic!("{:?} cannot be split", self);
        }
    }

    fn split(self, sp: Split) -> (Self, Self) {
        let Rectangle { w, h } = self;
        match sp {
            Split::Horizontal(y) => (Rectangle::new(w, h-y), Rectangle::new(w, y)),
            Split::Vertical(x) => (Rectangle::new(w-x, h), Rectangle::new(x, h)),
        }
    }
}

enum Split {
    Horizontal(usize),
    Vertical(usize),
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
