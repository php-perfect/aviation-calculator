use std::error::Error;
use std::fmt;

use enterpolation::{DiscreteGenerator, Generator, Sorted, SortedGenerator};
use enterpolation::utils::lerp;

struct TakeoffDistances<const COUNT: usize> {
    mass: Sorted<[f64; COUNT]>,
    takeoff_run: Sorted<[f64; COUNT]>,
    to_50_feet: Sorted<[f64; COUNT]>,
}

const _ROTAX_912_UL: TakeoffDistances<3> = TakeoffDistances {
    mass: Sorted::new_unchecked([472.5, 525.0, 540.0]),
    takeoff_run: Sorted::new_unchecked([106.0, 140.0, 147.0]),
    to_50_feet: Sorted::new_unchecked([265.0, 350.0, 367.0]),
};

const ROTAX_912_ULS: TakeoffDistances<5> = TakeoffDistances {
    mass: Sorted::new_unchecked([472.5, 525.0, 540.0, 570.0, 600.0]),
    takeoff_run: Sorted::new_unchecked([100.0, 128.0, 136.0, 141.0, 153.0]),
    to_50_feet: Sorted::new_unchecked([225.0, 320.0, 338.0, 352.0, 375.0]),
};

///
///
/// # Arguments
///
/// * `mass`:
/// * `pressure_altitude`:
/// * `temperature`:
/// * `slope`:
/// * `wet_surface`:
/// * `soft_surface`:
/// * `high_gras`:
///
/// returns: Result<(f64, f64), TakeoffParameterError>
///
/// # Examples
///
/// ```
///
/// ```
pub fn calculate_start_distance(mass: f64, pressure_altitude: f64, temperature: f64, slope: f64, wet_surface: bool, soft_surface: bool, high_gras: bool) -> Result<(f64, f64), TakeoffParameterError> {
    let takeoff_table = ROTAX_912_ULS;

    if mass < takeoff_table.mass.first().unwrap() {
        return Err(TakeoffParameterError("Mass is lower than the minimum available data".into()));
    }

    if mass > takeoff_table.mass.last().unwrap() {
        return Err(TakeoffParameterError("Mass is higher than the maximum available data".into()));
    }

    return Ok((
        round(apply_corrections(
            calculate_base_distance(mass, takeoff_table.mass, takeoff_table.takeoff_run),
            pressure_altitude,
            temperature,
            slope,
            wet_surface,
            soft_surface,
            high_gras,
        )?, 2),
        round(apply_corrections(
            calculate_base_distance(mass, takeoff_table.mass, takeoff_table.to_50_feet),
            pressure_altitude,
            temperature,
            slope,
            wet_surface,
            soft_surface,
            high_gras,
        )?, 2)
    ));
}

fn calculate_base_distance<const COUNT: usize>(mass: f64, masses: Sorted<[f64; COUNT]>, distances: Sorted<[f64; COUNT]>) -> f64 {
    let distance_graph = Generator::stack(masses, distances);
    let (min_index, max_index, factor) = masses.upper_border(mass);
    let min = distance_graph.gen(min_index).1;
    let max = distance_graph.gen(max_index).1;

    lerp(min, max, factor)
}

fn apply_corrections(mut takeoff_distance: f64, pressure_altitude: f64, temperature: f64, slope: f64, wet_surface: bool, soft_surface: bool, high_gras: bool) -> Result<f64, TakeoffParameterError> {
    if pressure_altitude < 0.0 {
        return Err(TakeoffParameterError("Pressure altitude must be greater than zero".into()));
    }

    takeoff_distance *= 1.0 + 0.1 * (pressure_altitude / 1000.0);
    takeoff_distance *= 1.0 + 0.01 * temperature;
    takeoff_distance *= 1.0 + 0.1 * slope;

    if wet_surface {
        takeoff_distance *= 1.1;
    }

    if soft_surface {
        takeoff_distance *= 1.5;
    }

    if high_gras {
        takeoff_distance *= 1.2;
    }

    Ok(takeoff_distance)
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
}

#[derive(Debug, Clone)]
pub struct TakeoffParameterError(String);

impl Error for TakeoffParameterError {}

impl fmt::Display for TakeoffParameterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uls_472_weight_too_low() {
        let result = calculate_start_distance(472.0, 0.0, 0.0, 0.0, false, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn uls_472_weight_too_high() {
        let result = calculate_start_distance(600.1, 0.0, 0.0, 0.0, false, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn uls_472_negative_pressure_altitude() {
        let result = calculate_start_distance(472.5, -0.1, 0.0, 0.0, false, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn uls_472() {
        let result = calculate_start_distance(472.5, 0.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472_temp() {
        let result = calculate_start_distance(472.5, 0.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472_pressure() {
        let result = calculate_start_distance(472.5, 3000.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (130.0, 292.5));
    }

    #[test]
    fn uls_472_pressure2() {
        let result = calculate_start_distance(472.5, 3200.5, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (132.01, 297.01));
    }

    #[test]
    fn uls_525() {
        let result = calculate_start_distance(525.0, 0.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (128.0, 320.0));
    }

    #[test]
    fn uls_525_temp() {
        let result = calculate_start_distance(525.0, 0.0, 3.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (131.84, 329.6));
    }

    #[test]
    fn uls_525_slope() {
        let result = calculate_start_distance(525.0, 0.0, 0.0, -2.2, false, false, false);
        assert_eq!(result.unwrap(), (99.84, 249.6));
    }

    #[test]
    fn uls_550() {
        let result = calculate_start_distance(550.0, 0.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (137.67, 342.67));
    }

    #[test]
    fn uls_600() {
        let result = calculate_start_distance(600.0, 0.0, 0.0, 0.0, false, false, false);
        assert_eq!(result.unwrap(), (153.0, 375.0));
    }

    #[test]
    fn uls_600_wet() {
        let result = calculate_start_distance(600.0, 0.0, 0.0, 0.0, true, false, false);
        assert_eq!(result.unwrap(), (168.3, 412.5));
    }

    #[test]
    fn uls_600_wet_and_soft() {
        let result = calculate_start_distance(600.0, 0.0, 0.0, 0.0, true, true, false);
        assert_eq!(result.unwrap(), (252.45, 618.75));
    }

    #[test]
    fn uls_600_combined() {
        let result = calculate_start_distance(600.0, 2000.5, -2.0, 3.0, true, true, true);
        assert_eq!(result.unwrap(), (463.15, 1135.18));
    }
}