use std::ops::{Add, Mul};

pub const FALL_SLIDE_LEFT: [Point; 5] = [
    point(0, -1),
    point(-1, -1),
    point(1, -1),
    point(-1, 0),
    point(1, 0),
];

pub const FALL_SLIDE_RIGHT: [Point; 5] = [
    point(0, -1),
    point(1, -1),
    point(-1, -1),
    point(1, 0),
    point(-1, 0),
];

pub const CLOSED_NEIGHBOURS: [Point; 9] = [
    point(-1, -1),
    point(-1, 0),
    point(-1, 1),
    point(0, 1),
    point(0, 0),
    point(0, -1),
    point(1, 1),
    point(1, 0),
    point(1, -1),
];

pub const FALL_TUMBLE_LEFT: [Point; 3] = [point(0, -1), point(-1, -1), point(1, -1)];

pub const FALL_TUMBLE_RIGHT: [Point; 3] = [point(0, -1), point(1, -1), point(-1, -1)];

pub const DOWN: Point = point(0, -1);
pub const UP: Point = point(0, 1);
pub const LEFT: Point = point(-1, 0);
pub const RIGHT: Point = point(1, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn index(self, y_size: usize) -> usize {
        let (x, y) = self.utup();
        x * y_size + y
    }

    pub fn utup(self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }

    pub fn tup(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<Point> for i32 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        point(rhs.x * self, rhs.y * self)
    }
}

pub const fn point(x: i32, y: i32) -> Point {
    Point { x, y }
}
