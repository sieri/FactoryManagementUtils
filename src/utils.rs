use num_traits::{Float, Num, NumCast, One, ToPrimitive};
use std::fmt::Display;
use std::ops::AddAssign;

///A trait for any number needed
pub trait Number:
    Num + PartialOrd + Copy + AddAssign + One + eframe::emath::Numeric + ToPrimitive + Display
{
}
impl<T> Number for T where
    T: Num
        + PartialOrd
        + Copy
        + AddAssign
        + One
        + eframe::emath::Numeric
        + NumCast
        + ToPrimitive
        + Display
{
}

///A trait for any float needed
pub trait FloatingNumber: Number + Float + From<f32> + PartialOrd + ToPrimitive {}
impl<T> FloatingNumber for T where T: Number + Float + From<f32> + NumCast + PartialOrd + ToPrimitive
{}

///A multi purpose enum to differentiate input from outputs
#[derive(serde::Deserialize, serde::Serialize, Debug, Eq, PartialEq)]
pub enum Io {
    Input,
    Output,
}

pub fn float_format<F: FloatingNumber>(float: F, precision: usize) -> String {
    let a = float.abs();
    let precision = if a >= F::one() {
        let n = (F::one() + a.log10().floor())
            .to_usize()
            .unwrap_or_else(|| {
                println!(
                    "Conversion failure of {} through {}",
                    float,
                    (F::one() + a.log10().floor())
                );
                0
            });
        if n <= precision {
            precision - n
        } else {
            0
        }
    } else if a > F::zero() {
        let n = (-(F::one() + a.log10().floor()))
            .to_usize()
            .unwrap_or_else(|| {
                println!(
                    "Conversion failure of {} through {}",
                    float,
                    (F::one() + a.log10().floor())
                );
                0
            });

        precision + n
    } else {
        0
    };
    format!("{float:.precision$}")
}

#[cfg(test)]
mod tests {
    use crate::utils::float_format;

    const TESTS_FORMATS: &[(f64, usize, &str)] = &[
        (0.000456, 2, "0.00046"),
        (0.043256, 3, "0.0433"),
        (0.01, 2, "0.010"),
        (10., 3, "10.0"),
        (1., 3, "1.00"),
        (456.789, 4, "456.8"),
    ];

    #[test]
    fn test_formatting_positive() {
        for (float, precision, expected) in TESTS_FORMATS {
            let result = float_format(*float, *precision);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn test_formatting_negative() {
        for (float, precision, expected) in TESTS_FORMATS {
            let result = float_format(-(*float), *precision);
            assert_eq!(result, format!("-{}", *expected));
        }
    }
}
