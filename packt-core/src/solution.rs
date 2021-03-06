use failure::Error;
use geometry::{Placement, Point, Rectangle, Rotation::*};
use problem::{Problem, Variant};
use std::fmt::{self, Formatter};
use std::iter;
use std::result;
use std::str::FromStr;
use std::time::Duration;

type Result<T, E = Error> = result::Result<T, E>;

#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    variant: Variant,
    allow_rotation: bool,
    source: Option<Problem>,
    placements: Vec<Placement>,
}

impl Solution {
    /// Checks whether this solution is valid.
    ///
    /// # Complexity
    ///
    /// Takes quadratic (in `self.placements.len()`) time.
    pub fn is_valid(&self) -> bool {
        if let Some((p1, p2)) = self
            .placements
            .iter()
            .enumerate()
            .flat_map(|(i, p)| iter::repeat(p).zip(self.placements.iter().skip(i + 1)))
            .find(|(p1, p2)| p1.overlaps(p2))
        {
            eprintln!("Overlap found: {:#?} and {:#?}", p1, p2);
            false
        } else {
            true
        }
    }

    pub fn evaluate(&mut self, duration: Duration) -> Result<Evaluation> {
        if !self.is_valid() {
            bail!("Overlap in solution")
        }

        let container = self.container()?;
        let min_area = self.placements.iter_mut().map(|p| p.rectangle.area()).sum();
        let empty_area = container.area() as i64 - min_area as i64;
        let filling_rate = (min_area as f64 / container.area() as f64) as f32;

        if filling_rate > 1.0 {
            bail!("Undetected overlap in solution")
        }

        Ok(Evaluation {
            container,
            min_area,
            empty_area,
            filling_rate,
            duration,
        })
    }


    pub fn container(&self) -> Result<Rectangle> {
        use std::cmp::max;

        let (x, y) = self.placements.iter().fold((0, 0), |(x, y), p| {
            let tr = p.top_right;
            let x = max(x, tr.x);
            let y = max(y, tr.y);
            (x, y)
        });

        let (x, y) = (x + 1, y + 1);

        let p = self.source.as_ref().unwrap();
        let container = match p.variant {
            Variant::Fixed(k) if y > k => bail!(
                "Solution placements exceed problem bounds: top: {}, bound: {}",
                y,
                k
            ),
            Variant::Fixed(k) => Rectangle::new(x, k),
            _ => Rectangle::new(x, y),
        };

        Ok(container)
    }

    pub fn source(&mut self, p: Problem) {
        self.source = Some(p);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evaluation {
    pub container: Rectangle,
    pub min_area: u64,
    pub empty_area: i64,
    pub filling_rate: f32,
    pub duration: Duration,
}

impl fmt::Display for Evaluation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Evaluation {
            min_area,
            container,
            empty_area,
            filling_rate,
            duration,
        } = self;
        let bb_area = container.area();

        write!(
            f,
            "lower bound on area: {}\nbounding box: {}, area: {}\nunused area in bounding box: \
             {}\nfilling_rate: {:.2}\ntook {}.{:.3}s",
            min_area,
            container,
            bb_area,
            empty_area,
            filling_rate,
            duration.as_secs(),
            duration.subsec_millis(),
        )
    }
}

impl FromStr for Solution {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut parts = s.split("placement of rectangles").map(str::trim);

        let problem: Problem = parts
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file: unable to parse problem"))?
            .parse()?;

        let Problem {
            variant,
            allow_rotation,
            source,
            rectangles,
        } = problem;

        let n = rectangles.len();
        let placements: Vec<Placement> = parts
            .next()
            .ok_or_else(|| format_err!("Unexpected end of file: unable to parse placements"))?
            .lines()
            .map(|s| {
                let tokens: Vec<&str> = s.split_whitespace().collect();
                let result = match (allow_rotation, tokens.as_slice()) {
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
            .zip(rectangles.iter())
            .map(|(result, &r)| result.map(|(rot, coord)| Placement::new(r, rot, coord)))
            .collect::<Result<_, _>>()?;

        if placements.len() != n {
            bail!("Solution contains a different number of placements than rectangles");
        }

        Ok(Solution {
            variant,
            allow_rotation,
            source: None,
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

        let expected = Solution {
            variant: Variant::Fixed(22),
            allow_rotation: false,
            source: None,
            evaluation: None,
            placements: vec![
                Placement::new(r1, Normal, Point::new(0, 0)),
                Placement::new(r2, Normal, Point::new(24, 3)),
            ],
        };

        let input = "container height: fixed 22\nrotations allowed: no\nnumber of rectangles: \
                     6\n12 8\n10 9\nplacement of rectangles\n0 0\n24 3";

        let result: Solution = input.parse().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn validation() {
        let r = Rectangle::new(10, 9);

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
                variant: Variant::Fixed(22),
                allow_rotation: false,
                source: None,
                evaluation: None,
                placements,
            }
        };

        assert!(solution.is_valid());
        let p = Placement::new(r, Normal, Point::new(0, 0));

        solution.placements = vec![p; 10000];
        assert!(!solution.is_valid());
    }

}
