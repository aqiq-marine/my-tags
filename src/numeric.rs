use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

/// A finite numeric value in the language.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NumericValue {
    Int(i128),
    Nat(u64),
    Float(f64),
}

impl NumericValue {
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Int(_) => "Int",
            Self::Nat(_) => "Nat",
            Self::Float(_) => "Float",
        }
    }

    /// Converts a number to the representation used when widening it to Float.
    ///
    /// As with any conversion to f64, large integer values may lose precision.
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Int(value) => value as f64,
            Self::Nat(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    pub fn is_finite(self) -> bool {
        match self {
            Self::Float(value) => value.is_finite(),
            Self::Int(_) | Self::Nat(_) => true,
        }
    }

    /// Compares numeric values without first rounding integers to f64.
    ///
    /// Mixed integer/float comparisons must preserve the integer's exact value:
    /// converting an i128 or u64 to f64 first can make distinct values compare
    /// equal once they exceed f64's integer precision.
    pub fn partial_cmp_numeric(self, other: Self) -> Option<Ordering> {
        if !self.is_finite() || !other.is_finite() {
            return None;
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a.partial_cmp(&b),
            (Self::Nat(a), Self::Nat(b)) => a.partial_cmp(&b),
            (Self::Int(a), Self::Nat(b)) => a.partial_cmp(&(b as i128)),
            (Self::Nat(a), Self::Int(b)) => (a as i128).partial_cmp(&b),
            (Self::Int(integer), Self::Float(float)) => Some(cmp_i128_f64(integer, float)),
            (Self::Nat(integer), Self::Float(float)) => {
                Some(cmp_i128_f64(i128::from(integer), float))
            }
            (Self::Float(float), Self::Int(integer)) => {
                Some(cmp_i128_f64(integer, float).reverse())
            }
            (Self::Float(float), Self::Nat(integer)) => {
                Some(cmp_i128_f64(i128::from(integer), float).reverse())
            }
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(&b),
        }
    }

    pub fn negate(self) -> Option<Self> {
        match self {
            Self::Nat(0) => Some(Self::Int(0)),
            Self::Nat(value) => i128::from(value).checked_neg().map(Self::Int),
            Self::Int(value) => value.checked_neg().map(Self::Int),
            Self::Float(value) => {
                let result = -value;
                result.is_finite().then_some(Self::Float(result))
            }
        }
    }
}

/// Compares an i128 with a finite f64 without converting the integer to f64.
fn cmp_i128_f64(integer: i128, float: f64) -> Ordering {
    // i128::MAX as f64 rounds up to 2^127, which is outside the i128 range.
    // i128::MIN is exactly -2^127 and is representable as f64.
    let upper_exclusive = i128::MAX as f64;
    let lower_inclusive = i128::MIN as f64;

    if float >= upper_exclusive {
        return Ordering::Less;
    }
    if float < lower_inclusive {
        return Ordering::Greater;
    }

    // The range checks above make this float-to-int conversion non-saturating.
    // Truncation is exact for the integral part of a finite binary float.
    let truncated = float.trunc() as i128;
    match integer.cmp(&truncated) {
        Ordering::Equal if float.fract() > 0.0 => Ordering::Less,
        Ordering::Equal if float.fract() < 0.0 => Ordering::Greater,
        ordering => ordering,
    }
}

impl From<i64> for NumericValue {
    fn from(value: i64) -> Self { Self::Int(i128::from(value)) }
}
impl From<i128> for NumericValue {
    fn from(value: i128) -> Self { Self::Int(value) }
}
impl From<i8> for NumericValue {
    fn from(value: i8) -> Self { Self::Int(i128::from(value)) }
}
impl From<i16> for NumericValue {
    fn from(value: i16) -> Self { Self::Int(i128::from(value)) }
}
impl From<i32> for NumericValue {
    fn from(value: i32) -> Self { Self::Int(i128::from(value)) }
}
impl From<isize> for NumericValue {
    fn from(value: isize) -> Self { Self::Int(value as i128) }
}
impl From<u8> for NumericValue {
    fn from(value: u8) -> Self { Self::Nat(u64::from(value)) }
}
impl From<u16> for NumericValue {
    fn from(value: u16) -> Self { Self::Nat(u64::from(value)) }
}
impl From<u32> for NumericValue {
    fn from(value: u32) -> Self { Self::Nat(u64::from(value)) }
}
impl From<usize> for NumericValue {
    fn from(value: usize) -> Self { Self::Nat(value as u64) }
}
impl From<f32> for NumericValue {
    fn from(value: f32) -> Self { Self::Float(f64::from(value)) }
}
impl From<u64> for NumericValue {
    fn from(value: u64) -> Self { Self::Nat(value) }
}
impl From<f64> for NumericValue {
    fn from(value: f64) -> Self { Self::Float(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_mixed_integer_and_float_values_exactly() {
        let integer = 9_007_199_254_740_993_i128;
        let rounded_float = 9_007_199_254_740_992.0_f64;

        assert_eq!(
            NumericValue::Int(integer).partial_cmp_numeric(NumericValue::Float(rounded_float)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            NumericValue::Int(-integer).partial_cmp_numeric(NumericValue::Float(-rounded_float)),
            Some(Ordering::Less)
        );
        assert_eq!(
            NumericValue::Nat(u64::MAX)
                .partial_cmp_numeric(NumericValue::Float(18_446_744_073_709_551_616.0)),
            Some(Ordering::Less)
        );
    }

    #[test]
    fn compares_fractional_floats_around_integer_values() {
        assert_eq!(
            NumericValue::Int(1).partial_cmp_numeric(NumericValue::Float(1.5)),
            Some(Ordering::Less)
        );
        assert_eq!(
            NumericValue::Int(-1).partial_cmp_numeric(NumericValue::Float(-1.5)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            NumericValue::Float(1.5).partial_cmp_numeric(NumericValue::Int(1)),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn compares_values_at_i128_float_conversion_boundaries() {
        assert_eq!(
            NumericValue::Int(i128::MAX)
                .partial_cmp_numeric(NumericValue::Float(2.0_f64.powi(127))),
            Some(Ordering::Less)
        );
        assert_eq!(
            NumericValue::Int(i128::MIN)
                .partial_cmp_numeric(NumericValue::Float(-2.0_f64.powi(127))),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn non_finite_float_values_are_not_comparable() {
        assert_eq!(
            NumericValue::Float(f64::NAN).partial_cmp_numeric(NumericValue::Int(0)),
            None
        );
        assert_eq!(
            NumericValue::Float(f64::INFINITY).partial_cmp_numeric(NumericValue::Float(1.0)),
            None
        );
    }

    #[test]
    fn negation_handles_integer_boundaries() {
        assert_eq!(NumericValue::Nat(0).negate(), Some(NumericValue::Int(0)));
        assert_eq!(NumericValue::Nat(u64::MAX).negate(), Some(NumericValue::Int(-i128::from(u64::MAX))));
        assert_eq!(NumericValue::Int(i128::MIN).negate(), None);
    }
}
