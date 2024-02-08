use std::ops::{Add, Div, Mul, Sub};

use iced::{Point, Vector};

#[derive(Debug, Clone, Copy, Default)]
pub struct NRVec {
    pub x: f32,
    pub y: f32,
}

pub fn nr_vec(x: f32, y: f32) -> NRVec {
    NRVec { x, y }
}

impl From<Vector> for NRVec {
    fn from(value: Vector) -> Self {
        NRVec {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Point> for NRVec {
    fn from(value: Point) -> Self {
        NRVec {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<NRVec> for Point {
    fn from(value: NRVec) -> Self {
        Point {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<NRVec> for Vector {
    fn from(value: NRVec) -> Self {
        Vector {
            x: value.x,
            y: value.y,
        }
    }
}

impl<T> Add<T> for NRVec
where
    T: Into<NRVec>,
{
    type Output = NRVec;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        NRVec {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub<T> for NRVec
where
    T: Into<NRVec>,
{
    type Output = NRVec;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        NRVec {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for NRVec
where
    T: Into<f32>,
{
    type Output = NRVec;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        NRVec {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div<T> for NRVec
where
    T: Into<f32>,
{
    type Output = NRVec;
    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        NRVec {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[test]
#[allow(unused)]
fn cunty_types() {
    let a = Point::new(0.0, 0.0);
    let b: NRVec = a.into();
    let c: Vector = b.into();
    let d: Point = b.into();
    let e = b + b;
    let f = b + e;
    let g = e - d;
    let h: NRVec = NRVec::from(c);
}
