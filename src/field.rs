pub mod finite_field;
pub mod real;

use boundy::Where;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

/// A trait alias that allows adding two instances or two references.
pub trait AddAny:
    Sized
    + Add<Output = Self>
    + for<'a, 'b> Where<&'a Self, Implements: Add<&'b Self, Output = Self>>
{
}

/// Blanket implementation for `AddAny`.
impl<T> AddAny for T where
    T: Sized
        + Add<Output = Self>
        + for<'a, 'b> Where<&'a Self, Implements: Add<&'b Self, Output = Self>>
{
}

/// Same as `AddAny`, but for subtraction.
pub trait SubAny:
    Sized
    + Sub<Output = Self>
    + for<'a, 'b> Where<&'a Self, Implements: Sub<&'b Self, Output = Self>>
{
}

/// Blanket implementation for `SubAny`.
impl<T> SubAny for T where
    T: Sized
        + Sub<Output = Self>
        + for<'a, 'b> Where<&'a Self, Implements: Sub<&'b Self, Output = Self>>
{
}

/// Same as `AddAny`, but for multiplication.
pub trait MulAny:
    Sized
    + Mul<Output = Self>
    + for<'a, 'b> Where<&'a Self, Implements: Mul<&'b Self, Output = Self>>
{
}

/// Blanket implementation for `MulAny`.
impl<T> MulAny for T where
    T: Sized
        + Mul<Output = Self>
        + for<'a, 'b> Where<&'a Self, Implements: Mul<&'b Self, Output = Self>>
{
}

/// Same as `AddAny`, but for division.
pub trait DivAny:
    Sized
    + Div<Output = Self>
    + for<'a, 'b> Where<&'a Self, Implements: Div<&'b Self, Output = Self>>
{
}

/// Blanket implementation for `DivAny`.
impl<T> DivAny for T where
    T: Sized
        + Div<Output = Self>
        + for<'a, 'b> Where<&'a Self, Implements: Div<&'b Self, Output = Self>>
{
}

/// A trait alias that allows performing standard arithmetic on two instances or
/// two references.
pub trait ArithAny: AddAny + SubAny + MulAny + DivAny {}

/// Blanket implementation for `ArithAny`.
impl<T> ArithAny for T where T: AddAny + SubAny + MulAny + DivAny {}

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
pub trait Field: ArithAny + PartialEq + Clone + Debug {
    /// The additive identity of the field.
    fn zero() -> Self;
    /// The multiplicative identity of the field.
    fn one() -> Self;
    /// The additive inverse of `self`.
    fn add_inv(&self) -> Self;
    /// The multiplicative inverse of `self`.
    fn mul_inv(&self) -> Self;
    /// The square root of `self` if it exists, None otherwise.
    fn sqrt(&self) -> Option<Self>;

    /// Subtract `rhs` is a shorthand for adding the addititve inverse of `rhs`.
    fn sub(self, rhs: Self) -> Self {
        self + rhs.add_inv()
    }

    /// Dividing by `rhs` is a shorthand for multiplying by the multiplicative
    /// inverse of `rhs`.
    fn div(self, rhs: Self) -> Self {
        self * rhs.mul_inv()
    }
}
