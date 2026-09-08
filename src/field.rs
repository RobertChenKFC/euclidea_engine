pub mod finite_field;

use std::ops::{Add, Mul};

/// A (mathematical) field, in layman terms, is essentially a group of elements
/// that satisfies the following:
///   - Has + and * operators
///   - Has an additive identity 0 (such that x + 0 = x)
///   - Has a multiplicative identity 0 (such that x * 1 = x)
///   - Every element x has an additive inverse -x (such that x + (-x) = 0)
///   - Every non-zero element x has a multiplicative inverse x**(-1) such that
///     x * (x**(-1)) = 1
/// Since this field will be used to compute circle and line intersections,
/// which involve quadratic polynomials, we also require that the field
/// implement a square root function. That is, for each element x, try to find
/// the element sqrt(x) such that sqrt(x) * sqrt(x) = x.
pub trait Field: Add<Output = Self> + Mul<Output = Self> + Eq + Sized {
    /// The additive identity of the field.
    fn zero() -> Self;
    /// The multiplicative identity of the field.
    fn one() -> Self;
    /// The additive inverse of `self`.
    fn add_inv(self) -> Self;
    /// The multiplicative inverse of `self`.
    fn mul_inv(self) -> Self;
    /// The square root of `self` if it exists, None otherwise.
    fn sqrt(self) -> Option<Self>;
}
