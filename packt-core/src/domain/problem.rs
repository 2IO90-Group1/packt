use domain::Rectangle;
use failure::Error;
use rand::{self, seq};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub variant: Variant,
    pub rotation_allowed: bool,
    pub rectangles: Vec<Rectangle>,
}

impl Problem {
    /// Generates a problem definition with an known solution by splitting `r`
    /// into `n` rectangles.
    ///
    ///# Panics
    ///
    /// This function will panic if  `n` is greater than `r.width * r.height`.
    // TODO: Add rotated rectangles, random variant
    // TODO: introduce builder
    fn generate_from(r: Rectangle, n: usize, v: Variant, rot: bool) -> Problem {
        let a = r.width * r.height;
        if n > a {
            panic!("{:?} cannot be split into {} rectangles", r, n)
        } else if n == a {
            let rectangles = vec![Rectangle::new(1, 1); n];
            return Problem {
                variant: v,
                rotation_allowed: rot,
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
            rotation_allowed: rot,
            rectangles,
        }
    }
}

impl fmt::Display for Problem {
    //noinspection RsTypeCheck
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = format!(
            "container height: {v}\nrotations allowed: {r}\nnumber of \
             rectangles: {n}\n",
            v = self.variant,
            r = if self.rotation_allowed {
                "yes"
            } else {
                "no"
            },
            n = self.rectangles.len()
        );

        let rstrings = self.rectangles
            .iter()
            .map(|r| format!("{}", r))
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}{}", s, rstrings)
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

        let rotation_allowed = match l2 {
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
            rotation_allowed,
            rectangles,
        })
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
            rotation_allowed: false,
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
