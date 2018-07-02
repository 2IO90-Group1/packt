use self::Rotation::*;
use failure::Error;
use rand::distributions::{IndependentSample, Normal};
use rand::{self, Rng};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
    area: u64,
}

impl Rectangle {
    fn split(self, sp: Cut) -> (Rectangle, Rectangle) {
        let Rectangle {
            width: w,
            height: h,
            ..
        } = self;

        match sp {
            Cut::Horizontal(y) => (Rectangle::new(w, h - y), Rectangle::new(w, y)),
            Cut::Vertical(x) => (Rectangle::new(w - x, h), Rectangle::new(x, h)),
        }
    }

    pub fn gen_with_area(area: u64) -> Rectangle {
        let divisors = (1..=(area as f64).sqrt() as u64)
            .into_iter()
            .filter(|i| area % i == 0)
            .collect::<Vec<u64>>();

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
        let width = width as u32;
        let height = height as u32;

        Rectangle {
            width,
            height,
            area,
        }
    }

    pub fn simple_rsplit(self) -> (Rectangle, Rectangle) {
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

    pub fn area(&self) -> u64 {
        self.area
    }

    pub fn new(width: u32, height: u32) -> Rectangle {
        Rectangle {
            width,
            height,
            area: height as u64 * width as u64,
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
        let result = match s.split_whitespace().collect::<Vec<&str>>().as_slice() {
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
    pub rectangle: Rectangle,
    pub rotation: Rotation,
    pub bottom_left: Point,
    pub top_right: Point,
}

impl Placement {
    pub fn new(r: Rectangle, rotation: Rotation, bottom_left: Point) -> Placement {
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

    pub fn overlaps(&self, rhs: &Placement) -> bool {
        rhs.bottom_left.y <= self.top_right.y
            && rhs.bottom_left.x <= self.top_right.x
            && self.bottom_left.y <= rhs.top_right.y
            && self.bottom_left.x <= rhs.top_right.x
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn overlap_detection() {
        let p1 = Placement::new(Rectangle::new(5, 5), Rotation::Normal, Point::new(0, 0));
        let p2 = Placement::new(Rectangle::new(5, 5), Rotation::Normal, Point::new(3, 3));
        assert!(p1.overlaps(&p2))
    }
}
