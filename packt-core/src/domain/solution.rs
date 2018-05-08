use domain::{Placement, Point, Problem, Rotation::*};
use failure::Error;
use std::iter;
use std::str::FromStr;

// TODO: consider taking over part of `Problem`s fields instead
#[derive(Clone, Debug, PartialEq)]

pub struct Solution {
    problem: Problem,
    placements: Vec<Placement>,
}

impl Solution {
    /// Checks whether this solution is valid.
    ///
    /// # Complexity
    ///
    /// Takes quadratic (in `self.placements.len()`) time.
    pub fn is_valid(&self) -> bool {
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

                let result = match (problem.allow_rotation, tokens.as_slice()) {
                    (false, [x, y]) => {
                        let p = Point::new(x.parse()?, y.parse()?);

                        (Normal, p)
                    }
                    (true, [rot, x, y]) => {
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
    use domain::{problem::Variant, Rectangle};
    use std::iter;

    #[test]
    fn solution_parsing() {
        let r1 = Rectangle::new(12, 8);
        let r2 = Rectangle::new(10, 9);

        let problem = Problem {
            variant: Variant::Fixed(22),
            allow_rotation: false,
            rectangles: vec![r1, r2],
            source: None,
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
            allow_rotation: false,
            rectangles: rectangles.clone(),
            source: None,
        };

        let mut coord = Point::new(0, 0);
        let placements = iter::repeat(r)
            .take(10000)
            .map(|r| {
                let result = Placement::new(r, Normal, coord);

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
        let p = Placement::new(r, Normal, Point::new(0, 0));
        solution.placements = vec![p; 10000];
        assert!(!solution.is_valid());
    }

}
