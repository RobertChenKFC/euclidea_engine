use crate::field::Field;
use crate::field::real::Real;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A `Complex` is similiar to complex numbers in math, and can be constructed
/// from any `Real`. This follows the standard construction of encoding a
/// complex number as a pair of `Real`s: one for the real part and one for the
/// imaginary part.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Complex<T: Real>(pub T, pub T);

impl<T: Real> Field for Complex<T> {
    fn zero() -> Self {
        Self(0_f32.into(), 0_f32.into())
    }

    fn one() -> Self {
        Self(1_f32.into(), 0_f32.into())
    }

    fn add_inv(&self) -> Self {
        Self(-&self.0, -&self.1)
    }

    fn mul_inv(&self) -> Self {
        //   1 / (a + bi)
        // = (a - bi) / ((a + bi)(a - bi))
        // = (a - bi) / (a**2 + b**2).
        let Self(a, b) = self;
        let mag_inv = (a * a + b * b).mul_inv();
        Self(a * &mag_inv, -b * mag_inv)
    }

    fn sqrt(&self) -> Option<Self> {
        // Let
        // - w = sqrt(x**2 + y**2)
        // - u = sqrt((w + x) / 2)
        // - v = y / 2u
        // then
        //   (u + vi)**2
        // = (u**2 - v**2) + 2uvi
        // = (w + x) / 2 - y**2 / 2(w + x) + (2 * u * y / 2u)i
        // = (w + x) / 2 - y**2 / 2(sqrt(x**2 + y**2) + x) + yi
        // = (w + x) / 2 - y**2(sqrt(x**2 + y**2) - x) / 2y**2 + yi
        // = (w + x) / 2 - (sqrt(x**2 + y**2) - x) / 2 + yi
        // = (sqrt(x**2 + y**2) + x) / 2 - sqrt(x**2 + y**2 - x) / 2 + yi
        // = x + yi.
        //
        // If x >= 0, this is what we compute since there will be no
        // catastrophic cancelation on w + x; if x < 0, we instead compute
        // - v = sgn(y) * sqrt((w - x) / 2)
        // - u = |y| / 2v
        // since there will be no catastrophic cancelation on w - x.
        let Self(x, y) = self;
        let w = x.hypot(y);
        let zero = T::zero();
        if *x >= zero {
            let u = ((&w + x) * 0.5_f32.into()).sqrt()?;
            let v = y / &(&u * &2_f32.into());
            Some(Self(u, v))
        } else {
            let v = ((&w - x) * 0.5_f32.into()).sqrt()?;
            let v = if *y >= zero { v } else { -v };
            let y = if *y >= zero { y } else { &-y };
            let u = y / &(&v * &2_f32.into());
            Some(Self(u, v))
        }
    }
}

impl<T: Real> Add for Complex<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        &self + &rhs
    }
}

impl<'a, 'b, T: Real> Add<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    fn add(self, rhs: &'b Complex<T>) -> Self::Output {
        let Complex(x1, y1) = self;
        let Complex(x2, y2) = rhs;
        Complex(x1 + x2, y1 + y2)
    }
}

impl<T: Real> Sub for Complex<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        &self - &rhs
    }
}

impl<'a, 'b, T: Real> Sub<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    fn sub(self, rhs: &'b Complex<T>) -> Self::Output {
        let Complex(x1, y1) = self;
        let Complex(x2, y2) = rhs;
        Complex(x1 - x2, y1 - y2)
    }
}

impl<T: Real> Mul for Complex<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        &self * &rhs
    }
}

impl<'a, 'b, T: Real> Mul<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    fn mul(self, rhs: &'b Complex<T>) -> Self::Output {
        //   (x1 + y1 i) * (x2 + y2 i)
        // = (x1x2 - y1y2) + (x1y2 + x2y1)i
        let Complex(x1, y1) = &self;
        let Complex(x2, y2) = &rhs;
        Complex(x1 * x2 - y1 * y2, x1 * y2 + x2 * y1)
    }
}

impl<T: Real> Div for Complex<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        &self / &rhs
    }
}

impl<'a, 'b, T: Real> Div<&'b Complex<T>> for &'a Complex<T> {
    type Output = Complex<T>;

    fn div(self, rhs: &'b Complex<T>) -> Self::Output {
        //   (x1 + y1 i) / (x2 + y2 i)
        // = ((x1x2 + y1y2) + (x2y1 - x1y2)i) / (x2**2 + y2**2)
        let Complex(x1, y1) = &self;
        let Complex(x2, y2) = &rhs;
        let mag_inv = (x2 * x2 + y2 * y2).mul_inv();
        Complex(
            &(x1 * x2 + y1 * y2) * &mag_inv,
            (x2 * y1 - x1 * y2) * mag_inv,
        )
    }
}

impl<T: Real> Neg for Complex<T> {
    type Output = Self;

    fn neg(self) -> Self {
        -&self
    }
}

impl<'a, T: Real> Neg for &'a Complex<T> {
    type Output = Complex<T>;

    fn neg(self) -> Self::Output {
        let Complex(x, y) = self;
        Complex(-x, -y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let x = Complex(1., 2.);
        let y = Complex(3., 4.);
        let z = Complex(4., 6.);
        assert_eq!(x + y, z);
    }

    #[test]
    fn test_sub() {
        let x = Complex(1., -2.);
        let y = Complex(-3., 4.);
        let z = Complex(4., -6.);
        assert_eq!(x - y, z);
    }

    fn ulp_error(x: f32, y: f32) -> u32 {
        if (x < 0.) != (y < 0.) {
            ulp_error(x, 0.) + ulp_error(0., y)
        } else {
            let x = x.abs().to_bits();
            let y = y.abs().to_bits();
            x.abs_diff(y)
        }
    }

    fn check_complex_numbers_close(num1: &Complex<f32>, num2: &Complex<f32>) {
        const ULP_THRESH: u32 = 1;

        let Complex(x1, y1) = num1;
        let Complex(x2, y2) = num2;
        let dist = ulp_error(*x1, *x2).max(ulp_error(*y1, *y2));
        if dist > ULP_THRESH {
            panic!(
                "Check failed: {:?} and {:?} differ by {} ULPs, exceeding \
                 threshold {}",
                num1, num2, dist, ULP_THRESH
            );
        }
    }

    #[test]
    fn test_sqrt() {
        let neg_one = -Complex::<f32>::one();
        let i = neg_one.sqrt();
        assert!(i.is_some());
        let i = i.unwrap();

        // DEBUG
        println!("-1: {:?}", neg_one);
        println!("i: {:?}", i);
        check_complex_numbers_close(&(i * i), &neg_one);

        let forty_five_degrees = i.sqrt();
        assert!(forty_five_degrees.is_some());
        let forty_five_degrees = forty_five_degrees.unwrap();

        // DEBUG
        println!("45 degrees: {:?}", forty_five_degrees);

        check_complex_numbers_close(
            &(forty_five_degrees * forty_five_degrees),
            &i,
        );

        let sixty_degrees = Complex(1_f32, 3_f32.sqrt());
        let thirty_degrees = sixty_degrees.sqrt();
        assert!(thirty_degrees.is_some());
        let thirty_degrees = thirty_degrees.unwrap();
        check_complex_numbers_close(
            &(thirty_degrees * thirty_degrees),
            &sixty_degrees,
        );
    }

    #[test]
    fn test_div() {
        let x = Complex(1_f32, 2_f32);
        let y = Complex(3_f32, 4_f32);
        check_complex_numbers_close(&(x / y * y), &x);
        check_complex_numbers_close(&(x * y.mul_inv() * y), &x);
    }
}
