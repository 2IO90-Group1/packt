use std::str::FromStr;

use failure::Error;

use self::Rotation::*;
use std::iter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Variant {
    Free,
    Fixed(usize),
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

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    variant: Variant,
    rotations_allowed: bool,
    rectangles: Vec<Rectangle>,
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

        let rotations_allowed = match l2 {
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
            rotations_allowed,
            rectangles,
        })
    }
}

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
            && self.bottom_left.y <= rhs.top_right.y
            && rhs.bottom_left.x <= self.top_right.x
            && self.bottom_left.x <= rhs.top_right.x
    }
}

// TODO: consider taking over part of `Problem`s fields instead
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    problem: Problem,
    placements: Vec<Placement>,
}

impl Solution {
    fn is_valid(&self) -> bool {
        self.placements
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                iter::repeat(p).zip(self.placements.iter().skip(i + 1))
            })
            .all(|(p1, p2)| !p1.overlaps(p2))
    }
}

impl FromStr for Solution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut parts = s.split("placement of rectangles")
            .map(str::trim);

        let problem: Problem = parts
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file"))?
            .parse()?;

        let placements: Vec<&str> = parts
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file"))?
            .lines()
            .collect();

        if placements.len() != problem.rectangles.len() {
            bail!(
                "Solution contains a different number of placements than \
                 rectangles"
            );
        }

        let placements: Vec<Placement> = placements
            .into_iter()
            .map(|s| {
                let tokens: Vec<&str> = s.split_whitespace().collect();
                let result = match tokens.as_slice() {
                    [x, y] if !problem.rotations_allowed => {
                        let p = Point::new(x.parse()?, y.parse()?);
                        (Normal, p)
                    }
                    [rot, x, y] if problem.rotations_allowed => {
                        let p = Point::new(x.parse()?, y.parse()?);
                        (rot.parse()?, p)
                    }
                    _ => bail!("Invalid format: {}", tokens.join(" ")),
                };

                Ok(result)
            })
            .zip(problem.rectangles.iter())
            .map(|(result, &r)| {
                result.map(|(rot, coord)| Placement::new(r, rot, coord))
            })
            .collect::<Result<_, _>>()?;

        Ok(Solution {
            problem,
            placements,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn problem_parsing() {
        let expected = Problem {
            variant: Variant::Fixed(22),
            rotations_allowed: false,
            rectangles: vec![Rectangle::new(12, 8), Rectangle::new(10, 9)],
        };
        let input = "container height: fixed 22\nrotations allowed: \
                     no\nnumber of rectangles: 6\n12 8\n10 9";
        let result: Problem = input.parse().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn solution_parsing() {
        let r1 = Rectangle::new(12, 8);
        let r2 = Rectangle::new(10, 9);
        let problem = Problem {
            variant: Variant::Fixed(22),
            rotations_allowed: false,
            rectangles: vec![r1, r2],
        };

        let expected = {
            Solution {
                problem,
                placements: vec![
                    Placement::new(r1, Normal, Point::new(0, 0)),
                    Placement::new(r2, Normal, Point::new(24, 3)),
                ],
            }
        };

        let input = "container height: fixed 22\nrotations allowed: \
                     no\nnumber of rectangles: 6\n12 8\n10 9\nplacement of \
                     rectangles\n0 0\n24 3";
        let result: Solution = input.parse().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn validation() {
        let r = Rectangle::new(10, 9);
        let rectangles = vec![r; 10000];
        let problem = Problem {
            variant: Variant::Fixed(22),
            rotations_allowed: false,
            rectangles: rectangles.clone(),
        };

        let mut coord = Point::new(0, 0);
        let placements = iter::repeat(r)
            .take(10000)
            .map(|r| {
                let result = Placement::new(r, Rotation::Normal, coord);
                coord.x += 11;
                result
            })
            .collect();

        let mut solution = {
            Solution {
                problem,
                placements,
            }
        };

        assert!(solution.is_valid());

        solution.placements =
            iter::repeat(Placement::new(r, Normal, Point::new(0, 0)))
                .take(10000)
                .collect();
        assert!(!solution.is_valid());
    }
}
