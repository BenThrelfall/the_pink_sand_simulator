use std::ops::Add;

pub const FALL_SLIDE_LEFT: [Point; 5] = [
    point(0, 1),
    point(-1, 1),
    point(1, 1),
    point(-1, 0),
    point(1, 0),
];

pub const FALL_SLIDE_RIGHT: [Point; 5] = [
    point(0, 1),
    point(1, 1),
    point(-1, 1),
    point(1, 0),
    point(-1, 0),
];

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn tup(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point { x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

pub const fn point(x: i32, y: i32) -> Point {
    Point { x, y }
}
