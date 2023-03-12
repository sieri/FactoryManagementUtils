use crate::utils::FloatingNumber;
use log::warn;

pub fn float_format<F: FloatingNumber>(float: F, precision: usize) -> String {
    let a = float.abs();
    let precision = if a >= F::one() {
        let n = (F::one() + a.log10().floor())
            .to_usize()
            .unwrap_or_else(|| {
                warn!(
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
                warn!(
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
    use crate::utils::formatting::float_format;
    use crate::utils::test_env;

    const TESTS_FORMATS: &[(f64, usize, &str)] = &[
        (0.000456, 2, "0.00046"),
        (0.043256, 3, "0.0433"),
        (0.01, 2, "0.010"),
        (10., 3, "10.0"),
        (1., 3, "1.00"),
        (456.789, 4, "456.8"),
        (156.723, 3, "157"),
    ];

    #[test]
    fn test_formatting_positive() {
        test_env::setup();
        for (float, precision, expected) in TESTS_FORMATS {
            let result = float_format(*float, *precision);
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn test_formatting_negative() {
        test_env::setup();
        for (float, precision, expected) in TESTS_FORMATS {
            let result = float_format(-(*float), *precision);
            assert_eq!(result, format!("-{}", *expected));
        }
    }
}
