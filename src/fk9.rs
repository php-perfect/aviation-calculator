use std::error::Error;
use std::fmt;

use enterpolation::{DiscreteGenerator, Generator, Sorted, SortedGenerator};
use enterpolation::utils::lerp;

struct TakeoffDistances {
    mass: Sorted<Vec<f64>>,
    takeoff_run: Sorted<Vec<f64>>,
    to_50_feet: Sorted<Vec<f64>>,
}

pub enum Engine {
    Rotax912Ul,
    Rotax912Uls,
}

///
///
/// # Arguments
///
/// * `engine`: ROTAX 912 UL or ROTAX 912 ULS
/// * `mass`: Mass in kg
/// * `pressure_altitude`: Pressure altitude in ft
/// * `temperature`: Temperature deviation in °C from default temperature for the given pressure altitude
/// * `slope`: Slope (positive or negative) in percent
/// * `wet_surface`: Wet surface
/// * `soft_surface`: Soft surface
/// * `high_gras`: Gras higher than 3cm
///
/// returns: Result<(f64, f64), TakeoffParameterError> Takeoff run, to 50 ft Height
///
/// # Examples
///
/// ```
/// use aviation_calculator::fk9::*;
/// use aviation_calculator::fk9::Engine::Rotax912Uls;
///
/// let distances: (f64, f64) = calculate_start_distance(Rotax912Uls, 525.0, 100.0, 0.8, 0.0, true, false, false).unwrap();
/// ```
pub fn calculate_start_distance(
    engine: Engine,
    mass: f64,
    pressure_altitude: f64,
    temperature: f64,
    slope: f64,
    wet_surface: bool,
    soft_surface: bool,
    high_gras: bool,
) -> Result<(f64, f64), TakeoffParameterError> {
    let takeoff_table = takeoff_distances_by_engine(engine);

    if mass < takeoff_table.mass.first().unwrap() {
        return Err(TakeoffParameterError(
            "Mass is lower than the minimum available data".into(),
        ));
    }

    if mass > takeoff_table.mass.last().unwrap() {
        return Err(TakeoffParameterError(
            "Mass is higher than the maximum available data".into(),
        ));
    }

    return Ok((
        round(
            apply_corrections(
                calculate_base_distance(mass, &takeoff_table.mass, &takeoff_table.takeoff_run),
                pressure_altitude,
                temperature,
                slope,
                wet_surface,
                soft_surface,
                high_gras,
            )?,
            2,
        ),
        round(
            apply_corrections(
                calculate_base_distance(mass, &takeoff_table.mass, &takeoff_table.to_50_feet),
                pressure_altitude,
                temperature,
                slope,
                wet_surface,
                soft_surface,
                high_gras,
            )?,
            2,
        ),
    ));
}

fn takeoff_distances_by_engine(engine: Engine) -> TakeoffDistances {
    match engine {
        Engine::Rotax912Ul => TakeoffDistances {
            mass: Sorted::new_unchecked(vec![472.5, 525.0, 540.0]),
            takeoff_run: Sorted::new_unchecked(vec![106.0, 140.0, 147.0]),
            to_50_feet: Sorted::new_unchecked(vec![265.0, 350.0, 367.0]),
        },
        Engine::Rotax912Uls => TakeoffDistances {
            mass: Sorted::new_unchecked(vec![472.5, 525.0, 540.0, 570.0, 600.0]),
            takeoff_run: Sorted::new_unchecked(vec![100.0, 128.0, 136.0, 141.0, 153.0]),
            to_50_feet: Sorted::new_unchecked(vec![225.0, 320.0, 338.0, 352.0, 375.0]),
        },
    }
}

fn calculate_base_distance(
    mass: f64,
    masses: &Sorted<Vec<f64>>,
    distances: &Sorted<Vec<f64>>,
) -> f64 {
    let distance_graph = Generator::stack(masses, distances);
    let (min_index, max_index, factor) = masses.upper_border(mass);
    let min = distance_graph.gen(min_index).1;
    let max = distance_graph.gen(max_index).1;

    lerp(min, max, factor)
}

fn apply_corrections(
    mut takeoff_distance: f64,
    pressure_altitude: f64,
    temperature: f64,
    slope: f64,
    wet_surface: bool,
    soft_surface: bool,
    high_gras: bool,
) -> Result<f64, TakeoffParameterError> {
    takeoff_distance =
        apply_environmental_corrections(takeoff_distance, pressure_altitude, temperature)?;
    takeoff_distance = apply_surface_corrections(
        takeoff_distance,
        slope,
        wet_surface,
        soft_surface,
        high_gras,
    );

    Ok(takeoff_distance)
}

fn apply_surface_corrections(
    mut takeoff_distance: f64,
    slope: f64,
    wet_surface: bool,
    soft_surface: bool,
    high_gras: bool,
) -> f64 {
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

    takeoff_distance
}

fn apply_environmental_corrections(
    mut takeoff_distance: f64,
    pressure_altitude: f64,
    temperature: f64,
) -> Result<f64, TakeoffParameterError> {
    if pressure_altitude < 0.0 {
        return Err(TakeoffParameterError(
            "Pressure altitude must be greater than zero".into(),
        ));
    }

    takeoff_distance *= 1.0 + 0.1 * (pressure_altitude / 1000.0);
    takeoff_distance *= 1.0 + 0.01 * temperature;

    Ok(takeoff_distance)
}

fn round(x: f64, decimals: u8) -> f64 {
    let y = 10_i32.pow(decimals.into()) as f64;
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
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_472_weight_too_high() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            600.1,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_472_negative_pressure_altitude() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.5,
            -0.1,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_472() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn ul_472() {
        let result = calculate_start_distance(
            Engine::Rotax912Ul,
            472.5,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (106.0, 265.0));
    }

    #[test]
    fn uls_472_temp() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472_pressure() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.5,
            3000.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (130.0, 292.5));
    }

    #[test]
    fn uls_472_pressure2() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            472.5,
            3200.5,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (132.01, 297.01));
    }

    #[test]
    fn uls_525() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (128.0, 320.0));
    }

    #[test]
    fn uls_525_temp() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            3.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (131.84, 329.6));
    }

    #[test]
    fn uls_525_slope() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            0.0,
            -2.2,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (99.84, 249.6));
    }

    #[test]
    fn uls_550() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            550.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (137.67, 342.67));
    }

    #[test]
    fn uls_600() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (153.0, 375.0));
    }

    #[test]
    fn uls_600_wet() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            0.0,
            0.0,
            true,
            false,
            false,
        );
        assert_eq!(result.unwrap(), (168.3, 412.5));
    }

    #[test]
    fn uls_600_wet_and_soft() {
        let result =
            calculate_start_distance(Engine::Rotax912Uls, 600.0, 0.0, 0.0, 0.0, true, true, false);
        assert_eq!(result.unwrap(), (252.45, 618.75));
    }

    #[test]
    fn uls_600_combined() {
        let result = calculate_start_distance(
            Engine::Rotax912Uls,
            600.0,
            2000.5,
            -2.0,
            3.0,
            true,
            true,
            true,
        );
        assert_eq!(result.unwrap(), (463.15, 1135.18));
    }
}
