use domain::Rectangle;
use failure::Error;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Variant {
    Free,
    Fixed(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub variant: Variant,
    pub rotation_allowed: bool,
    pub rectangles: Vec<Rectangle>,
}

impl Problem {
    //    fn generate
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
    use super::*;

    #[test]
    fn parsing() {
        let expected = Problem {
            variant: Variant::Fixed(22),
            rotation_allowed: false,
            rectangles: vec![Rectangle::new(12, 8), Rectangle::new(10, 9)],
        };
        let input = "container height: fixed 22\nrotations allowed: \
                     no\nnumber of rectangles: 6\n12 8\n10 9";
        let result: Problem = input.parse().unwrap();
        assert_eq!(result, expected);
    }
}
