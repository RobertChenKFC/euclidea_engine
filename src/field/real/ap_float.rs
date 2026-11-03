use crate::field::Field;
use crate::field::real::Real;
use rug::Float;
use rug::float::ParseFloatError;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

/// A wrapper for arbitrary precision float from Rug (wrapper of MPFR). All
/// instances of `APFloat<P>` is gauranteed to have `P` bits of precision.
#[derive(Clone, PartialEq, PartialOrd)]
struct APFloat<const P: u32>(Float);

impl<const P: u32> Debug for APFloat<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<const P: u32> Field for APFloat<P> {
    fn zero() -> Self {
        Self(Float::with_val(P, 0))
    }

    fn one() -> Self {
        Self(Float::with_val(P, 1))
    }

    fn add_inv(&self) -> Self {
        Self(-self.0.clone())
    }

    fn mul_inv(&self) -> Self {
        Self(self.0.clone().recip())
    }

    fn sqrt(&self) -> Option<Self> {
        Some(Self(self.0.clone().sqrt()))
    }
}

impl<const P: u32> Add<APFloat<P>> for APFloat<P> {
    type Output = APFloat<P>;

    fn add(self, rhs: APFloat<P>) -> Self::Output {
        APFloat(self.0 + rhs.0)
    }
}

impl<'a, 'b, const P: u32> Add<&'b APFloat<P>> for &'a APFloat<P> {
    type Output = APFloat<P>;

    fn add(self, rhs: &'b APFloat<P>) -> Self::Output {
        APFloat(Float::with_val(P, &self.0 + &rhs.0))
    }
}

impl<const P: u32> Sub<APFloat<P>> for APFloat<P> {
    type Output = APFloat<P>;

    fn sub(self, rhs: APFloat<P>) -> Self::Output {
        APFloat(self.0 - rhs.0)
    }
}

impl<'a, 'b, const P: u32> Sub<&'b APFloat<P>> for &'a APFloat<P> {
    type Output = APFloat<P>;

    fn sub(self, rhs: &'b APFloat<P>) -> Self::Output {
        APFloat(Float::with_val(P, &self.0 - &rhs.0))
    }
}

impl<const P: u32> Mul<APFloat<P>> for APFloat<P> {
    type Output = APFloat<P>;

    fn mul(self, rhs: APFloat<P>) -> Self::Output {
        APFloat(self.0 * rhs.0)
    }
}

impl<'a, 'b, const P: u32> Mul<&'b APFloat<P>> for &'a APFloat<P> {
    type Output = APFloat<P>;

    fn mul(self, rhs: &'b APFloat<P>) -> Self::Output {
        APFloat(Float::with_val(P, &self.0 * &rhs.0))
    }
}

impl<const P: u32> Div<APFloat<P>> for APFloat<P> {
    type Output = APFloat<P>;

    fn div(self, rhs: APFloat<P>) -> Self::Output {
        APFloat(self.0 / rhs.0)
    }
}

impl<'a, 'b, const P: u32> Div<&'b APFloat<P>> for &'a APFloat<P> {
    type Output = APFloat<P>;

    fn div(self, rhs: &'b APFloat<P>) -> Self::Output {
        APFloat(Float::with_val(P, &self.0 / &rhs.0))
    }
}

impl<const P: u32> Neg for APFloat<P> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.add_inv()
    }
}

impl<'a, const P: u32> Neg for &'a APFloat<P> {
    type Output = APFloat<P>;

    fn neg(self) -> Self::Output {
        self.add_inv()
    }
}

impl<const P: u32> Real for APFloat<P> {
    fn hypot(&self, other: &Self) -> Self {
        Self(self.0.clone().hypot(&other.0))
    }
}

impl<const P: u32> From<f32> for APFloat<P> {
    fn from(value: f32) -> Self {
        Self(Float::with_val(P, value))
    }
}

impl<const P: u32> Into<f64> for APFloat<P> {
    fn into(self) -> f64 {
        self.0.to_f64()
    }
}

impl<const P: u32> FromStr for APFloat<P> {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(APFloat(Float::with_val(P, Float::parse(s)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tests for this struct are not meant to be very comprehensive, they
    // are just testing that the plumbing for rug::Float is working.

    type F100 = APFloat<100>;

    #[test]
    fn test_add() {
        let x = F100::from(1.);
        let y = F100::from(2.);
        let z: f64 = (&x + &y).into();
        assert_eq!(z, 3.);
        let z: f64 = (x + y).into();
        assert_eq!(z, 3.);
    }

    #[test]
    fn test_sub() {
        let x = F100::from(3.);
        let y = F100::from(4.);
        let z: f64 = (&x - &y).into();
        assert_eq!(z, -1.);
        let z: f64 = (x - y).into();
        assert_eq!(z, -1.);
    }

    #[test]
    fn test_mul() {
        let x = F100::from(5.);
        let y = F100::from(6.);
        let z: f64 = (&x * &y).into();
        assert_eq!(z, 30.);
        let z: f64 = (x * y).into();
        assert_eq!(z, 30.);
    }

    #[test]
    fn test_div() {
        let x = F100::from(7.);
        let y = F100::from(8.);
        let z: f64 = (&x / &y).into();
        assert_eq!(z, 0.875);
        let z: f64 = (x / y).into();
        assert_eq!(z, 0.875);
    }

    /// A fast algorithm to compute pi: https://en.wikipedia.org/wiki/Chudnovsky_algorithm
    ///   \pi = 4270934400/(sqrt(10005)*s)
    /// where
    ///   s   = \sum_{k=0}^\infty(p_k/q_k)
    ///   p_k = (-1)^k (6k)! (ak+b)
    ///   q_k = (3k)! (k!)^3 c^(3k)
    ///   a   = 545140134
    ///   b   = 13591409
    ///   c   = 640320
    /// Since
    ///   q_{k+1}/q_k = (3k+1)(3k+2)(3k+3)(k+1)^3 c^3
    /// If we keep track of our running sum of s as a numerator-denominator
    /// pair, we can keep multiplying our pair by q_{k+1}/q_k to match the
    /// denomintor of the next pair, so we can keep the pairs as integers
    /// and keep accumulating terms to them.
    fn chudnovsky_algorithm<R: Real>(num_terms: usize) -> R {
        let a = R::from_str("545140134")
            .expect("Integer should be parsed successfully");
        let b = R::from_str("13591409")
            .expect("Integer should be parsed succesfully");
        let c = R::from_str("640320")
            .expect("Integer should be parsed succesfully");
        let c3 = (&c * &c) * c;
        let three = R::from(3.);
        let six = R::from(6.);

        let mut s_num = R::zero();
        let mut num = R::one();
        let mut s_den = R::one();
        let mut ak_plus_b = b;

        for k in 0..num_terms {
            s_num = s_num + &num * &ak_plus_b;

            let k = R::from(k as f32);
            num = num.add_inv();
            for i in 1..=6 {
                let i = R::from(i as f32);
                num = num * (&six * &k + i);
            }

            let mut mult = c3.clone();
            let k_plus_1 = &k + &R::one();
            for i in 1..=3 {
                let i = R::from(i as f32);
                mult = &mult * &k_plus_1 * (&three * &k + i);
            }
            s_num = &s_num * &mult;
            s_den = &s_den * &mult;
            ak_plus_b = &ak_plus_b + &a;
        }

        let den_scale = R::from_str("4270934400")
            .expect("Integer should be parsed succesfully");
        let num_scale = R::from_str("10005")
            .expect("Integer should be parsed succesfully")
            .sqrt()
            .expect("Positive integer should have square root");
        let den = den_scale * s_den;
        let num = num_scale * s_num;
        den / num
    }

    #[test]
    fn test_pi() {
        // First 100 digits of pi (101 digits if we include the leading 3).
        let reference = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";
        // Chudnovsky's algorithm produces ~14 decimal digits per term, so we
        // need ceil(101/14) = 8 terms.
        let num_terms = 8;
        // To store 100 digits of pi, we need
        //   ceil(log2(10^101)) = ceil(101*log2(10)) = 336 bits.
        const P: u32 = 336;
        let result = chudnovsky_algorithm::<APFloat<P>>(num_terms);
        // Convert to string and extract the 100 digits of pi (including the leading 3 and the decimal point).
        let result = &format!("{:?}", result)[0..102];
        assert_eq!(reference, result);
    }
}
