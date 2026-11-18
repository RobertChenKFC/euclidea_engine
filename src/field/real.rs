pub mod ap_float;
pub mod float;

use crate::field::Field;
use boundy::Where;
use std::fmt::Debug;
use std::str::FromStr;

/// A `Real` is a `Field` that is similar to real numbers in math, but can be
/// converted from/to floating point numbers. Certain methods of floating point
/// numbers are also exposed in this trait so they can preserve higher precision
/// rather than be composed of many elementary operations and subject to
/// rounding errors.
trait Real:
    Field + From<f32> + Into<f64> + FromStr + Where<Self::Err, Implements: Debug>
{
    fn hypot(&self, other: &Self) -> Self;
}
