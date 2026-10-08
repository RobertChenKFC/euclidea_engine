use crate::field::Field;
use std::ops::{Add, Mul};

/// A finite field is a field with finite number (P) of elements. We have some
/// restrictions on P to make it easy to compute the inverses and square roots
/// of the field elements (and also to make sure that it's actually a field):
/// - We require that P is a prime number (for easy multiplicative inverse)
/// - We require P = 3 (mod 4) (for easy square root)
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct FiniteField<const P: u64>(u64);

/// Standard power function computing (x ** p) % m.
/// TODO: duplicate this code for constant and non-constant m. Constant m allows
/// for the compiler to optimize away the integer division. Unfortunately, since
/// this function is const (since the following is_prime function is also
/// const), and Rust currently does not allow trait methods to be const, we
/// don't have an elegant way to avoid code duplication (macros is the only
/// known way, which is not very elegant). Come back to optimize this when
/// performance becomes a bottleneck.
const fn pow(x: u64, p: u64, m: u64) -> u64 {
    let mut mask = 1u64 << (u64::BITS - 1);
    let mut result: u128 = 1;
    let x = x as u128;
    let m = m as u128;
    while mask != 0 {
        result = (result * result) % m;
        if (p & mask) != 0 {
            result = (result * x) % m;
        }
        mask >>= 1;
    }
    // Since result is mod m, and m is u64, result can never overflow u64.
    result as u64
}

/// Check if `x` is prime. Uses the deterministic version of the Rabin-Miller
/// Test by checking for a fixed list of bases (see `ARRAY` below) instead of
/// random testing. This is sufficent for all u64 numbers according to this
/// math stackexchange post:
/// https://math.stackexchange.com/questions/2481148/primality-testing-for-64-bit-numbers
const fn is_prime(x: u64) -> bool {
    // x - 1 = d * 2**s.
    let s = (x - 1).trailing_zeros();
    let d = (x - 1) >> s;

    // For 64-bit unsigned integers, testing these numbers for pseudo-primality
    // is sufficient for actual primality.
    const ARRAY: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    let mut i = 0;
    while i < ARRAY.len() {
        let a = ARRAY[i];
        if a >= x {
            break;
        }

        // In order to be a pseudoprime, we must have either:
        // (1) a**d == 1 (mod x)
        let mut result = pow(a, d, x) as u128;
        let mut is_pseudoprime = result == 1;
        if !is_pseudoprime {
            // Or we have
            // (2) a**(d * 2**r) = -1 (mod x) for some 0 <= r < s
            let mut r = 0;
            let x = x as u128;
            while r < s {
                if result == x - 1 {
                    is_pseudoprime = true;
                    break;
                }
                r += 1;
                result = (result * result) % x;
            }
        }
        if !is_pseudoprime {
            return false;
        }
        i += 1;
    }
    true
}

impl<const P: u64> FiniteField<P> {
    const CHECK_3MOD4: () = assert!(P % 4 == 3);
    const CHECK_PRIME: () = assert!(is_prime(P));
    pub fn new(x: u64) -> Self {
        let _ = Self::CHECK_3MOD4;
        let _ = Self::CHECK_PRIME;
        Self(x)
    }
}

impl<const P: u64> Field for FiniteField<P> {
    fn zero() -> Self {
        Self(0)
    }

    fn one() -> Self {
        Self(1)
    }

    fn add_inv(self) -> Self {
        Self(P - self.0)
    }

    fn mul_inv(self) -> Self {
        // By Fermat's Little Theorem,
        //   x**(p-1) = 1 (mod p),
        // so
        //   x * (x**(p-2)) = 1 (mod p)
        Self(pow(self.0, P - 2, P))
    }

    fn sqrt(self) -> Option<Self> {
        // By Euler's Criterion, we have
        //   x**((p - 1) / 2) = 1 (mod p)
        // if x is a quadratic residue (ie. has a square root). Since we
        // restricted to finite fields such that p = 3 (mod 4), (p + 1) / 4 is
        // an integer, thus
        //     x
        //   = x * 1
        //   = x * (x**((p - 1) / 2)
        //   = x**((p + 1) / 2)
        //   = (x**((p + 1) / 4))**2 (mod p)
        let s = Self(pow(self.0, (P + 1) / 4, P));
        if s * s == self { Some(s) } else { None }
    }
}

impl<const P: u64> Add for FiniteField<P> {
    type Output = FiniteField<P>;

    fn add(self, rhs: FiniteField<P>) -> FiniteField<P> {
        let a: u128 = self.0.into();
        let b: u128 = rhs.0.into();
        let p: u128 = P.into();
        Self(((a + b) % p).try_into().expect(
            "Since the sum is mod P and P is u64, the product can never \
             overflow u64",
        ))
    }
}

impl<const P: u64> Mul for FiniteField<P> {
    type Output = FiniteField<P>;

    fn mul(self, rhs: FiniteField<P>) -> FiniteField<P> {
        let a: u128 = self.0.into();
        let b: u128 = rhs.0.into();
        let p: u128 = P.into();
        Self(((a * b) % p).try_into().expect(
            "Since the product is mod P and P is u64, the product can never \
             overflow u64",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert!(is_prime(23));
        // 169 = 13 * 13
        assert!(!is_prime(169));
        // The largest 32-bit prime number
        assert!(is_prime(4294967291));
        // The largest 64-bit prime number
        assert!(is_prime(18446744073709551557));
        // The 64-bit prime number with the largest prime factor
        assert!(!is_prime(18446744030759878681));
    }

    #[test]
    fn test_finite_field_primality() {
        let t = trybuild::TestCases::new();
        t.pass("src/field/finite_field_prime_test.rs");
        t.compile_fail("src/field/finite_field_composite_fail_test.rs");
    }

    #[test]
    fn test_finite_field_3mod4() {
        let t = trybuild::TestCases::new();
        t.pass("src/field/finite_field_3mod4_test.rs");
        t.compile_fail("src/field/finite_field_1mod4_test.rs");
    }

    #[test]
    fn test_id() {
        type F = FiniteField<18446744073709551427>;
        let arr: [u64; _] = [
            1214737953024963560,
            10577622953116021983,
            10244583881964291772,
            6873164487734729602,
            5120771050695524446,
            9906548044868202751,
            13949989876848520052,
            6819277698622686316,
            17308903799572649432,
            12831357654994941331,
        ];
        let zero = F::zero();
        let one = F::one();
        for x in arr {
            let x = F::new(x);
            assert_eq!(x, x + zero);
            assert_eq!(x, x * one);
        }
    }

    #[test]
    fn test_inv() {
        type F = FiniteField<18446744073709551427>;
        let arr: [u64; _] = [
            4500084517760998543,
            2276445815203296925,
            27262290558880751,
            5899183762289935604,
            5380524916130629900,
            15078895074288113300,
            3567356482340476093,
            16273465769900126905,
            6428085164277861831,
            16813150760807897358,
        ];
        let zero = F::zero();
        let one = F::one();
        for x in arr {
            let x = F::new(x);
            assert_eq!(x + x.add_inv(), zero);
            assert_eq!(x * x.mul_inv(), one);
        }
    }

    #[test]
    fn test_sqrt() {
        type F = FiniteField<18446744073709551427>;
        let arr: [(u64, bool); _] = [
            (6829073795728943971, true),
            (6864166158447497996, false),
            (15314932630331613284, true),
            (17062160704620153557, true),
            (15188948528448276324, true),
            (10316744378332773192, false),
            (2217734309646047493, false),
            (660468764307669089, false),
            (7832709915471985544, false),
            (9785736128487807008, true),
        ];
        for (x, has_sqrt) in arr {
            let x = F::new(x);
            let s = x.sqrt();
            match s {
                Some(s) => {
                    assert!(has_sqrt);
                    assert_eq!(s * s, x);
                }
                None => {
                    assert!(!has_sqrt);
                }
            }
        }
    }
}
