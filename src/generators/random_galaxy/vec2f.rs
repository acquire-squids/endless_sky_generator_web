use crate::wandom::XoShiRo256SS;

use std::{cmp::Ordering, ops};

crate::macros::wasm_newtype! {
    in vec2f =>
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub Vec2f ;
    pub x: f64,
    pub y: f64,
}

impl Vec2f {
    pub fn rand_from_rect(rng: &mut XoShiRo256SS, a: Self, b: Self) -> Self {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        Self {
            x: (rng.rand_range(0, (a.x - b.x).abs() as u64) as f64) + a.x.min(b.x),
            y: (rng.rand_range(0, (a.y - b.y).abs() as u64) as f64) + a.y.min(b.y),
        }
    }

    #[must_use]
    pub fn normalize(&self) -> Self {
        let hypot = self.x.hypot(self.y);

        let hypot = if hypot.is_normal() { hypot } else { 1.0 };

        Self {
            x: self.x / hypot,
            y: self.y / hypot,
        }
    }

    #[must_use]
    pub const fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[must_use]
    pub fn distance(&self, other: Self) -> f64 {
        let difference = *self - other;

        difference.x.hypot(difference.y)
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[must_use]
    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }

    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn average(v: &[Self]) -> Self {
        Self::sum(v) / (v.len().max(1) as f64)
    }

    #[must_use]
    pub fn sum(v: &[Self]) -> Self {
        v.iter().fold(Self::zero(), |accum, v| accum + *v)
    }

    #[must_use]
    pub fn attraction(v: &[Self]) -> Self {
        match v.len().cmp(&1) {
            Ordering::Greater => {
                let max_pull_magnitude = v.iter().fold(0.0f64, |accum, v| accum.max(v.magnitude()));

                let max_pull_magnitude = if max_pull_magnitude.is_normal() {
                    max_pull_magnitude
                } else {
                    1.0
                };

                v.iter()
                    .map(|v| v.normalize() * (1.0 - (v.magnitude() / max_pull_magnitude)))
                    .fold(Self::zero(), |accum, v| accum + v)
            }
            Ordering::Equal => v[0],
            Ordering::Less => Self::zero(),
        }
    }
}

#[must_use]
pub fn intersects((a1, a2): (Vec2f, Vec2f), (b1, b2): (Vec2f, Vec2f)) -> bool {
    let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) =
        ((a1.x, a1.y), (a2.x, a2.y), (b1.x, b1.y), (b2.x, b2.y));

    let t = (x1 - x3).mul_add(y3 - y4, -((y1 - y3) * (x3 - x4)));

    let u = (x1 - x2).mul_add(y1 - y3, -((y1 - y2) * (x1 - x3)));

    let d = (x1 - x2).mul_add(y3 - y4, -((y1 - y2) * (x3 - x4)));

    let t = t / d;
    let u = -(u / d);

    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) && !t.is_nan() && !u.is_nan()
}

impl ops::Add for Vec2f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub for Vec2f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Mul<f64> for Vec2f {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Div<f64> for Vec2f {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
