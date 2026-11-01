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

    pub fn partial_cmp_numeric(self, other: Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a.partial_cmp(&b),
            (Self::Nat(a), Self::Nat(b)) => a.partial_cmp(&b),
            (Self::Int(a), Self::Nat(b)) => a.partial_cmp(&(b as i128)),
            (Self::Nat(a), Self::Int(b)) => (a as i128).partial_cmp(&b),
            (a, b) => a.as_f64().partial_cmp(&b.as_f64()),
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
