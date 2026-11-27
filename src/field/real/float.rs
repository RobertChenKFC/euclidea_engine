use crate::field::Field;
use crate::field::real::Real;

impl Field for f32 {
    fn zero() -> Self {
        0.
    }

    fn one() -> Self {
        1.
    }

    fn add_inv(&self) -> Self {
        -self
    }

    fn mul_inv(&self) -> Self {
        self.recip()
    }

    fn sqrt(&self) -> Option<Self> {
        Some(f32::sqrt(*self))
    }
}

impl Real for f32 {
    fn hypot(&self, other: &Self) -> Self {
        f32::hypot(*self, *other)
    }
}

impl Field for f64 {
    fn zero() -> Self {
        0.
    }

    fn one() -> Self {
        1.
    }

    fn add_inv(&self) -> Self {
        -self
    }

    fn mul_inv(&self) -> Self {
        self.recip()
    }

    fn sqrt(&self) -> Option<Self> {
        Some(f64::sqrt(*self))
    }
}

impl Real for f64 {
    fn hypot(&self, other: &Self) -> Self {
        f64::hypot(*self, *other)
    }
}
