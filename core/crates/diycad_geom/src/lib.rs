#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_constructor_sets_coordinates() {
        let p = Point2::new(1.5, -2.0);
        assert_eq!(p, Point2 { x: 1.5, y: -2.0 });
    }
}
