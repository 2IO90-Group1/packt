#[macro_use]
extern crate failure;

use std::str::FromStr;

use failure::Error;

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
pub struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn new(x: usize, y: usize) -> Coordinate {
        Coordinate { x, y }
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
        use Rotation::*;

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
    ll_coord: Coordinate,
}

impl Placement {
    fn new(
        rectangle: Rectangle,
        orientation: Rotation,
        ll_coord: Coordinate,
    ) -> Placement {
        Placement {
            rectangle,
            rotation: orientation,
            ll_coord,
        }
    }
}

// TODO: consider taking over part of `Problem`s fields instead
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    problem: Problem,
    placements: Vec<Placement>,
}

impl FromStr for Solution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use Rotation::*;

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

        let placements = placements
            .into_iter()
            .map(|s| {
                let tokens: Vec<&str> = s.split_whitespace().collect();
                let result = match tokens.as_slice() {
                    [x, y] => {
                        let coord = Coordinate::new(x.parse()?, y.parse()?);
                        (Rotation::Normal, coord)
                    }
                    [rot, x, y] => {
                        let coord = Coordinate::new(x.parse()?, y.parse()?);
                        (rot.parse()?, coord)
                    }
                    _ => bail!("Invalid format: {}", tokens.join(" ")),
                };

                Ok(result)
            })
            .zip(problem.rectangles.iter())
            .map(|(result, &r)| {
                let (rot, coord) = result?;
                if !problem.rotations_allowed && rot != Normal {
                    bail!(
                        "Rotations were not allowed, but solution contains \
                         rotated rectangles"
                    );
                }

                Ok(Placement::new(r, rot, coord))
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
                    Placement::new(r1, Rotation::Normal, Coordinate::new(0, 0)),
                    Placement::new(
                        r2,
                        Rotation::Normal,
                        Coordinate::new(24, 3),
                    ),
                ],
            }
        };

        let input = "container height: fixed 22\nrotations allowed: \
                     no\nnumber of rectangles: 6\n12 8\n10 9\nplacement of \
                     rectangles\n0 0\n24 3";
        let result: Solution = input.parse().unwrap();
        assert_eq!(result, expected);
    }
}
