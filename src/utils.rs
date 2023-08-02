use std::f64::consts::PI;

const FEET: f64 = 0.3048_f64; /* m */

/// # Convert meter to feet
///
/// ## Arguments
///
/// * `meter`: Value in meters
///
/// returns: f64 Value in feet
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let feet = meter_to_feet(50.0);
/// ```
pub fn meter_to_feet(meter: f64) -> f64 {
    meter / FEET
}

/// # Convert feet to meter
///
/// ## Arguments
///
/// * `feet`: Value in feet
///
/// returns: f64 Value in meter
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let meter = feet_to_meter(50.0);
/// ```
pub fn feet_to_meter(feet: f64) -> f64 {
    feet * FEET
}

/// # Convert to Degree
///
/// ## Arguments
///
/// * `value`: Value in radians
///
/// returns: f64 value in degrees
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let degree = to_degree(std::f64::consts::FRAC_PI_2);
/// ```
pub fn to_degree(value: f64) -> f64 {
    180.0_f64 / PI * value
}

/// # Convert to Radian
///
/// ## Arguments
///
/// * `value`: Value in degrees
///
/// returns: f64 Value in radians
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let radians = to_radian(90.0);
/// ```
pub fn to_radian(value: f64) -> f64 {
    PI / 180.0_f64 * value
}

/// # Normalize Degree
///
/// ## Arguments
///
/// * `value`: Any degree value
///
/// returns: f64 Representation of the degree value between 0 and 360
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let degree = normalize_degree(370.0);
/// ```
pub fn normalize_degree(value: f64) -> f64 {
    value % 360.0_f64
}

/// # Round
///
/// ## Arguments
///
/// * `number`: Number to round
/// * `precision`: Precision to round
///
/// returns: f64 Rounded value
///
/// # Examples
///
/// ```
/// use aviation_calculator::utils::*;
///
/// let rounded = round(55.5555, 2);
/// ```
pub fn round(number: f64, precision: u8) -> f64 {
    let base = 10_i32.pow(precision.into()) as f64;
    (number * base).round() / base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_to_feet_1() {
        let result = meter_to_feet(1.0);
        assert_eq!(result, 3.280839895013123);
    }

    #[test]
    fn meter_to_feet_5() {
        let result = meter_to_feet(5.5);
        assert_eq!(result, 18.04461942257218);
    }

    #[test]
    fn feet_to_meter_1() {
        let result = feet_to_meter(1.0);
        assert_eq!(result, 0.3048);
    }

    #[test]
    fn feet_to_meter_5() {
        let result = feet_to_meter(5.5);
        assert_eq!(result, 1.6764000000000001);
    }

    #[test]
    fn to_degree_1() {
        let result = to_degree(std::f64::consts::FRAC_PI_2);
        assert_eq!(result, 90.0);
    }

    #[test]
    fn to_radian_1() {
        let result = to_radian(90.0);
        assert_eq!(result, std::f64::consts::FRAC_PI_2);
    }

    #[test]
    fn normalize_degree_1() {
        let result = normalize_degree(370.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn normalize_degree_2() {
        let result = normalize_degree(360.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn round_1() {
        let result = round(55.5555, 2);
        assert_eq!(result, 55.56);
    }
}
