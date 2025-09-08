use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

// custom converters
//
pub fn str_to_dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

pub fn u128_to_dec<T>(num: T) -> Decimal
where
    u128: From<T>,
{
    Decimal::from_ratio(u128::from(num), 1)
}

/// A fixed-point decimal value with 18 fractional digits
/// Simplified version for Solana tests
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Decimal(u128);

#[derive(Debug, PartialEq, Eq)]
pub struct DecimalRangeExceeded;

impl fmt::Display for DecimalRangeExceeded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decimal range exceeded")
    }
}

impl std::error::Error for DecimalRangeExceeded {}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckedFromRatioError {
    DivideByZero,
    Overflow,
}

impl fmt::Display for CheckedFromRatioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "Division by zero"),
            Self::Overflow => write!(f, "Overflow in ratio calculation"),
        }
    }
}

impl std::error::Error for CheckedFromRatioError {}

impl Decimal {
    pub const DECIMAL_FRACTIONAL: u128 = 1_000_000_000_000_000_000u128; // 1*10^18
    pub const DECIMAL_PLACES: u32 = 18;

    /// Create a new Decimal from raw atomic units
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    /// Create a 1.0 Decimal
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Check if the decimal is zero
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Get the raw atomic units
    pub const fn atomics(&self) -> u128 {
        self.0
    }

    /// Convert x% into basis points
    pub const fn bps(&self) -> u16 {
        ((100 * self.atomics()) / Self::DECIMAL_FRACTIONAL) as u16
    }

    /// Convert x% into Decimal
    pub const fn percent(x: u64) -> Self {
        let atomics = (x as u128) * 10_000_000_000_000_000;
        Self(atomics)
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn from_ratio(numerator: u128, denominator: u128) -> Self {
        match Self::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal with error handling
    pub fn checked_from_ratio(
        numerator: u128,
        denominator: u128,
    ) -> Result<Self, CheckedFromRatioError> {
        if denominator == 0 {
            return Err(CheckedFromRatioError::DivideByZero);
        }

        // Calculate numerator * DECIMAL_FRACTIONAL / denominator
        // We need to be careful about overflow
        let numerator_u256 = numerator;
        let fractional_u256 = Self::DECIMAL_FRACTIONAL;
        let denominator_u256 = denominator;

        // Check for overflow in multiplication
        match numerator_u256.checked_mul(fractional_u256) {
            Some(product) => {
                let result = product / denominator_u256;
                Ok(Decimal(result))
            }
            None => {
                // Handle overflow by using higher precision arithmetic
                // This is a simplified approach - in practice you'd want proper u256 arithmetic
                if numerator > u128::MAX / Self::DECIMAL_FRACTIONAL {
                    Err(CheckedFromRatioError::Overflow)
                } else {
                    let result = (numerator * Self::DECIMAL_FRACTIONAL) / denominator;
                    Ok(Decimal(result))
                }
            }
        }
    }

    /// Convert to uint by truncating fractional part
    pub fn to_uint_floor(self) -> u128 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Convert to uint by rounding up
    pub fn to_uint_ceil(self) -> u128 {
        if self.0 == 0 {
            0
        } else {
            1 + ((self.0 - 1) / Self::DECIMAL_FRACTIONAL)
        }
    }
}

impl FromStr for Decimal {
    type Err = Box<dyn std::error::Error>;

    /// Parse a decimal string like "1.23", "1", "0.000001"
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap();
        let whole: u128 = whole_part.parse().map_err(|_| "Error parsing whole part")?;

        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .ok_or("Value too big")?;

        if let Some(fractional_part) = parts_iter.next() {
            if fractional_part.len() > Self::DECIMAL_PLACES as usize {
                return Err(format!(
                    "Cannot parse more than {} fractional digits",
                    Self::DECIMAL_PLACES
                )
                .into());
            }

            let fractional: u128 = fractional_part
                .parse()
                .map_err(|_| "Error parsing fractional part")?;

            let exp = Self::DECIMAL_PLACES - fractional_part.len() as u32;
            let fractional_factor = 10u128.pow(exp);

            atomics = atomics
                .checked_add(fractional * fractional_factor)
                .ok_or("Value too big")?;
        }

        if parts_iter.next().is_some() {
            return Err("Too many decimal points".into());
        }

        Ok(Decimal(atomics))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = self.0 / Self::DECIMAL_FRACTIONAL;
        let fractional = self.0 % Self::DECIMAL_FRACTIONAL;

        if fractional == 0 {
            write!(f, "{}", whole)
        } else {
            let fractional_string = format!("{:018}", fractional);
            let trimmed = fractional_string.trim_end_matches('0');
            write!(f, "{}.{}", whole, trimmed)
        }
    }
}

// Basic arithmetic operations
impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        // For multiplication: (a * b) / DECIMAL_FRACTIONAL
        // We need to handle potential overflow
        let result = (self.0 * other.0) / Self::DECIMAL_FRACTIONAL;
        Decimal(result)
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.0 == 0 {
            panic!("Division by zero");
        }
        let result = (self.0 * Self::DECIMAL_FRACTIONAL) / other.0;
        Decimal(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Decimal::from_str("1").unwrap(), Decimal::one());
        assert_eq!(Decimal::from_str("0").unwrap(), Decimal::zero());
        assert_eq!(
            Decimal::from_str("1.5").unwrap(),
            Decimal(1_500_000_000_000_000_000)
        );
        assert_eq!(
            Decimal::from_str("0.000000000000000001").unwrap(),
            Decimal(1)
        );
    }

    #[test]
    fn test_from_ratio() {
        assert_eq!(Decimal::from_ratio(1, 2), Decimal::from_str("0.5").unwrap());
        assert_eq!(
            Decimal::from_ratio(3, 4),
            Decimal::from_str("0.75").unwrap()
        );
        assert_eq!(Decimal::from_ratio(1, 1), Decimal::one());
    }

    #[test]
    fn test_display() {
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::from_str("1.5").unwrap().to_string(), "1.5");
        assert_eq!(Decimal::from_str("0.1").unwrap().to_string(), "0.1");
    }

    #[test]
    fn test_arithmetic() {
        let a = Decimal::from_str("1.5").unwrap();
        let b = Decimal::from_str("2.5").unwrap();

        assert_eq!(a + b, Decimal::from_str("4").unwrap());
        assert_eq!(b - a, Decimal::from_str("1").unwrap());
        assert_eq!(a * b, Decimal::from_str("3.75").unwrap());
        assert_eq!(b / a, Decimal::from_str("1.666666666666666666").unwrap());
    }

    #[test]
    fn test_bps() {
        assert_eq!(Decimal::from_str("5").unwrap().bps(), 500);
        assert_eq!(Decimal::from_str("0").unwrap().bps(), 0);
    }

    #[test]
    fn test_percent() {
        assert_eq!(Decimal::percent(50), Decimal::from_str("0.5").unwrap());
        assert_eq!(Decimal::percent(100), Decimal::one());
    }

    #[test]
    fn test_conversions() {
        let d = Decimal::from_str("12.345").unwrap();
        assert_eq!(d.to_uint_floor(), 12);
        assert_eq!(d.to_uint_ceil(), 13);

        let d = Decimal::from_str("12.0").unwrap();
        assert_eq!(d.to_uint_floor(), 12);
        assert_eq!(d.to_uint_ceil(), 12);
    }
}
