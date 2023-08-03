use enterpolation::{DiscreteGenerator, Generator, Sorted, SortedGenerator, utils::lerp};
use snafu::prelude::*;

use crate::meteorology::{calculate_temperature_deviation, UndefinedPressureAltitudeError};
use crate::utils::{feet_to_meter, round};

const MAX_TEMP: f64 = 70.0;
const MIN_TEMP: f64 = -90.0;
const MAX_SLOPE: f64 = 25.0;

#[derive(Debug)]
struct TakeoffDistances {
    mass: Sorted<Vec<f64>>,
    takeoff_run: Sorted<Vec<f64>>,
    to_50_feet: Sorted<Vec<f64>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Engine {
    Rotax912Ul,
    Rotax912Uls,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceCondition {
    Inconspicuous,
    Slush,
    Snow,
    PowderSnow,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GrassSurface {
    pub wet: bool,
    pub soft_ground: bool,
    pub damaged_turf: bool,
    pub high_grass: bool,
}

#[derive(Debug, Snafu)]
pub enum TakeoffCalculationError {
    #[snafu(display("Mass {mass} kg is below the minimum available data ({min} kg)"))]
    MassTooLow { min: f64, mass: f64 },

    #[snafu(display("Mass {mass} kg is above the maximum available data ({max} kg)"))]
    MassTooHigh { max: f64, mass: f64 },

    #[snafu(display("Temperature {temperature} °C is below the minimum sensible data ({min} °C)"))]
    TemperatureTooLow { min: f64, temperature: f64 },

    #[snafu(display("Temperature {temperature} °C is above the maximum sensible data ({max} °C)"))]
    TemperatureTooHigh { max: f64, temperature: f64 },

    #[snafu(display("Slope {slope} % is is too steep to provide sensible data (Maximum {max} %)"))]
    SlopeTooSteep { max: f64, slope: f64 },

    #[snafu(display("The given pressure altitude is not defined by the ICAO standard atmosphere: {source}"))]
    InvalidPressureAltitude { source: UndefinedPressureAltitudeError },
}

pub type TakeoffResult = Result<(f64, f64), TakeoffCalculationError>;

/// # Takeoff Calculation for FK9 Mk VI
/// Calculations are based on the approved Flight Manual as well as the FSM 3/75 "Einflüsse auf die Länge der Startstrecke".
///
/// ## Arguments
///
/// * `engine`:Engine of the aircraft one of ROTAX 912 UL or ROTAX 912 ULS
/// * `mass`: Mass of the aircraft in kg
/// * `pressure_altitude`: Pressure altitude in ft
/// * `temperature`: Temperature on the runway in °C
/// * `slope`: Slope (positive or negative) in percentage
/// * `grass_surface`: If grass runway, its condition
/// * `surface_condition`: General condition of the runway
///
/// returns: Result<(f64, f64), TakeoffCalculationError> Takeoff run, to 50 ft Height
///
/// # Examples
///
/// ```
/// use aviation_calculator::fk9::*;
/// use aviation_calculator::fk9::Engine::Rotax912Uls;
///
/// let distances: (f64, f64) = calculate_takeoff_distance(Rotax912Uls, 525.0, 100.0, 21.3, 0.0, None, SurfaceCondition::Inconspicuous).unwrap();
/// ```
pub fn calculate_takeoff_distance(
    engine: Engine,
    mass: f64,
    pressure_altitude: f64,
    temperature: f64,
    slope: f64,
    grass_surface: Option<GrassSurface>,
    surface_condition: SurfaceCondition,
) -> TakeoffResult {
    if temperature > MAX_TEMP {
        return Err(TakeoffCalculationError::TemperatureTooHigh { max: MAX_TEMP, temperature });
    } else if temperature < MIN_TEMP {
        return Err(TakeoffCalculationError::TemperatureTooLow { min: MIN_TEMP, temperature });
    }

    if slope > MAX_SLOPE || slope < -MAX_SLOPE {
        return Err(TakeoffCalculationError::SlopeTooSteep { max: MAX_SLOPE, slope });
    }

    let takeoff_table = takeoff_distances_by_engine(engine);
    let min: f64 = takeoff_table.mass.first().unwrap();
    let max: f64 = takeoff_table.mass.last().unwrap();

    if mass < min {
        return Err(TakeoffCalculationError::MassTooLow { min, mass });
    } else if mass > max {
        return Err(TakeoffCalculationError::MassTooHigh { max, mass });
    }

    Ok((apply_corrections(
        calculate_base_distance(mass, &takeoff_table.mass, &takeoff_table.takeoff_run),
        pressure_altitude,
        temperature,
        slope,
        grass_surface,
        surface_condition,
    )?, apply_corrections(
        calculate_base_distance(mass, &takeoff_table.mass, &takeoff_table.to_50_feet),
        pressure_altitude,
        temperature,
        slope,
        grass_surface,
        surface_condition,
    )?))
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

    lerp(min, max, factor) / 120.0 * 100.0
}

fn apply_corrections(
    mut takeoff_distance: f64,
    pressure_altitude: f64,
    temperature: f64,
    slope: f64,
    grass_surface: Option<GrassSurface>,
    surface_condition: SurfaceCondition,
) -> Result<f64, TakeoffCalculationError> {
    takeoff_distance = apply_environmental_corrections(takeoff_distance, pressure_altitude, temperature)?;
    takeoff_distance *= 1.0 + 0.1 * slope;

    if grass_surface.is_some() {
        takeoff_distance = apply_grass_surface_corrections(
            takeoff_distance,
            grass_surface.unwrap(),
        );
    }

    Ok(round(match surface_condition {
        SurfaceCondition::Inconspicuous => takeoff_distance,
        SurfaceCondition::Slush => takeoff_distance * 1.3,
        SurfaceCondition::Snow => takeoff_distance * 1.5,
        SurfaceCondition::PowderSnow => takeoff_distance * 1.25,
    }, 2))
}

fn apply_grass_surface_corrections(mut takeoff_distance: f64, grass_surface: GrassSurface) -> f64 {
    takeoff_distance *= 1.2;

    if grass_surface.wet {
        takeoff_distance *= 1.1;
    }

    if grass_surface.soft_ground {
        takeoff_distance *= 1.5;
    }

    if grass_surface.damaged_turf {
        takeoff_distance *= 1.1;
    }

    if grass_surface.high_grass {
        takeoff_distance *= 1.2;
    }

    takeoff_distance
}

fn apply_environmental_corrections(
    takeoff_distance: f64,
    pressure_altitude: f64,
    temperature: f64,
) -> Result<f64, TakeoffCalculationError> {
    let distance = apply_pressure_altitude_correction(takeoff_distance, pressure_altitude);
    let temperature_deviation = calculate_temperature_deviation_for_correction(pressure_altitude, temperature)?;

    Ok(apply_temperature_correction(distance, temperature_deviation))
}

fn calculate_temperature_deviation_for_correction(pressure_altitude: f64, temperature: f64) -> Result<f64, TakeoffCalculationError> {
    Ok(calculate_temperature_deviation(feet_to_meter(pressure_altitude), temperature.max(0.0)).context(InvalidPressureAltitudeSnafu)?)
}

fn apply_pressure_altitude_correction(takeoff_distance: f64, pressure_altitude: f64) -> f64 {
    let multiplier = if pressure_altitude > 3000.0 {
        0.18
    } else if pressure_altitude > 1000.0 {
        0.13
    } else {
        0.10
    };

    takeoff_distance * (1.0 + multiplier * (pressure_altitude / 1000.0)).max(1.0)
}

fn apply_temperature_correction(takeoff_distance: f64, temperature_deviation: f64) -> f64 {
    takeoff_distance * (1.0 + 0.01 * temperature_deviation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uls_472_weight_too_low() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.0,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
        assert_eq!("Mass 472 kg is below the minimum available data (472.5 kg)", result.unwrap_err().to_string());
    }

    #[test]
    fn uls_472_weight_too_high() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.1,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
        assert_eq!("Mass 600.1 kg is above the maximum available data (600 kg)", result.unwrap_err().to_string());
    }

    #[test]
    fn uls_472_pressure_altitude_too_low() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            520.0,
            -5000.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
        assert_eq!("The given pressure altitude is not defined by the ICAO standard atmosphere: The pressure altitude -1524 m is below the minimum defined (-1000 m) in the ICAO Standard Atmosphere", result.unwrap_err().to_string());
    }

    #[test]
    fn uls_472_negative_pressure_altitude() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            -0.5,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472_slush() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Slush,
        );
        assert_eq!(result.unwrap(), (130.0, 292.5));
    }

    #[test]
    fn uls_472_snow() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Snow,
        );
        assert_eq!(result.unwrap(), (150.0, 337.5));
    }

    #[test]
    fn uls_472_powder_snow() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::PowderSnow,
        );
        assert_eq!(result.unwrap(), (125.0, 281.25));
    }

    #[test]
    fn ul_472() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Ul,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (106.0, 265.0));
    }

    #[test]
    fn uls_472_temp() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (100.0, 225.0));
    }

    #[test]
    fn uls_472_pressure() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            3000.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (147.26, 331.33));
    }

    #[test]
    fn uls_472_pressure2() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            472.5,
            3200.5,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (167.6, 377.1));
    }

    #[test]
    fn uls_525() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (128.0, 320.0));
    }

    #[test]
    fn uls_525_temp() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            3.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (112.64, 281.6));
    }

    #[test]
    fn uls_525_slope() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            525.0,
            0.0,
            15.0,
            -2.2,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (99.84, 249.6));
    }

    #[test]
    fn uls_550() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            550.0,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (137.67, 342.67));
    }

    #[test]
    fn uls_600() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (153.0, 375.0));
    }

    #[test]
    fn uls_600_wet() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            15.0,
            0.0,
            Some(GrassSurface { wet: true, ..GrassSurface::default() }),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (168.3, 412.5));
    }

    #[test]
    fn uls_600_wet_and_soft() {
        let result =
            calculate_takeoff_distance(Engine::Rotax912Uls, 600.0, 0.0, 15.0, 0.0, Some(GrassSurface { wet: true, soft_ground: true, damaged_turf: false, high_grass: false }), SurfaceCondition::Inconspicuous);
        assert_eq!(result.unwrap(), (252.45, 618.75));
    }

    #[test]
    fn uls_600_combined() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            2000.5,
            -2.0,
            3.0,
            Some(GrassSurface { wet: true, soft_ground: true, damaged_turf: true, high_grass: true }),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (485.6, 1190.2));
    }

    #[test]
    fn uls_600_max_pressure_altitude() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            262467.1,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (11773.24, 28855.99));
    }

    #[test]
    fn uls_600_above_max_pressure_altitude() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            262467.2,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_600_min_pressure_altitude() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            -3280.8,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (143.05, 350.63));
    }

    #[test]
    fn uls_600_below_min_pressure_altitude() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            -3280.9,
            15.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_600_min_temperature() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            -90.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (130.05, 318.75));
    }

    #[test]
    fn uls_600_below_min_temperature() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            -90.1,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_600_max_temperature() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            70.0,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert_eq!(result.unwrap(), (237.15, 581.25));
    }

    #[test]
    fn uls_600_above_max_temperature() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            70.1,
            0.0,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_600_above_max_negative_slope() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            13.0,
            -25.1,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn uls_600_above_max_slope() {
        let result = calculate_takeoff_distance(
            Engine::Rotax912Uls,
            600.0,
            0.0,
            13.0,
            25.1,
            Some(GrassSurface::default()),
            SurfaceCondition::Inconspicuous,
        );
        assert!(result.is_err());
    }

    #[test]
    fn apply_corrections_fsm75_3_example1() {
        let result = apply_corrections(316.0, 600.0, -3.0, 0.0, None, SurfaceCondition::Snow);
        assert_eq!(result.unwrap(), 433.05); // 444
    }

    #[test]
    fn apply_corrections_fsm75_3_example2() {
        let result = apply_corrections(465.0, 2000.0, 1.0, 0.0, Some(GrassSurface {
            wet: true,
            soft_ground: false,
            damaged_turf: false,
            high_grass: false,
        }), SurfaceCondition::Slush);
        assert_eq!(result.unwrap(), 904.46); // 904
    }

    #[test]
    fn apply_corrections_fsm75_3_example3() {
        let result = apply_corrections(465.0, 1150.0, 35.0, 0.0, None, SurfaceCondition::Inconspicuous);
        assert_eq!(result.unwrap(), 653.61); // 653
    }

    #[test]
    fn apply_corrections_fsm75_3_example4() {
        let result = apply_corrections(465.0, 600.0, 28.0, 0.0, Some(GrassSurface {
            wet: true,
            soft_ground: false,
            damaged_turf: false,
            high_grass: false,
        }), SurfaceCondition::Slush);
        assert_eq!(result.unwrap(), 965.84); // 1002
    }

    #[test]
    fn pressure_altitude() {
        let result = apply_pressure_altitude_correction(465.0, 2000.0);
        assert_eq!(result, 585.9);
    }

    #[test]
    fn temperature_deviation_below_zero() {
        let result1 = calculate_temperature_deviation_for_correction(200.0, -3.0);
        let result2 = calculate_temperature_deviation_for_correction(200.0, 0.0);
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn temperature_deviation_fsm_75_3_example1() {
        let result = calculate_temperature_deviation_for_correction(600.0, -3.0);
        assert_eq!(result.unwrap(), -13.81, "Temperature deviation does not comply with example 1 of FSM 3/75, expected to be ~-14°C");
    }

    #[test]
    fn temperature_deviation_fsm_75_3_example2() {
        let result = calculate_temperature_deviation_for_correction(2000.0, 1.0);
        assert_eq!(result.unwrap(), -10.04, "Temperature deviation does not comply with example 2 of FSM 3/75, expected to be ~-10°C");
    }

    #[test]
    fn temperature_deviation_fsm_75_3_example3() {
        let result = calculate_temperature_deviation_for_correction(1150.0, 35.0);
        assert_eq!(result.unwrap(), 22.28, "Temperature deviation does not comply with example 3 of FSM 3/75, expected to be ~22°C");
    }

    #[test]
    fn temperature_deviation_fsm_75_3_example4() {
        let result = calculate_temperature_deviation_for_correction(600.0, 28.0);
        assert_eq!(result.unwrap(), 14.19, "Temperature deviation does not comply with example 4 of FSM 3/75, expected to be ~14°C");
    }

    #[test]
    fn apply_temperature_correction_negative() {
        let result = apply_temperature_correction(120.0, -10.0);
        assert_eq!(result, 108.0);
    }

    #[test]
    fn apply_temperature_correction_neutral() {
        let result = apply_temperature_correction(120.0, 0.0);
        assert_eq!(result, 120.0);
    }

    #[test]
    fn apply_temperature_correction_positive() {
        let result = apply_temperature_correction(120.0, 10.0);
        assert_eq!(result, 132.0);
    }
}
