// https://www.dwd.de/DE/service/lexikon/begriffe/S/Standardatmosphaere_pdf.pdf;jsessionid=0749F4152999FCCDFB9CD32F32A3B866.live21073?__blob=publicationFile&v=3

use std::error::Error;
use std::fmt;

const ISA_TEMPERATURE: f64 = 288.15_f64; /* K */
const ISA_PRESSURE: f64 = 1013.25_f64; /* hPa */
const TROPOSPHERIC_TEMPERATURE_LAPSE: f64 = 0.0065_f64; /* K m-1 */
const STRATOSPHERIC_TEMPERATURE_LAPSE: f64 = 0.0010_f64; /* K m-1 */
const SPECIFIC_GAS_CONSTANT: f64 = 287.058_f64;
const GRAVITATIONAL_ACCELERATION: f64 = 9.81_f64; /* m/s */
const ICAO_MINIMUM_ELEVATION: f64 = -1_000.0_f64; /* m */
const ICAO_MAXIMUM_ELEVATION: f64 = 80_000.0_f64; /* m */

#[derive(Debug, Clone)]
pub struct UndefinedError(String);

impl Error for UndefinedError {}

impl fmt::Display for UndefinedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct AtmosphericLevel {
    base: u32,
    lapse_rate: f64,
    base_temperature: f64,
}

const LEVELS: [&AtmosphericLevel; 4] = [
    // Troposphere
    &AtmosphericLevel {
        base: 0,
        lapse_rate: TROPOSPHERIC_TEMPERATURE_LAPSE,
        base_temperature: 15.0,
    },
    // Tropopause
    &AtmosphericLevel {
        base: 11_000,
        lapse_rate: 0.0,
        base_temperature: -56.5,
    },
    // Stratosphere
    &AtmosphericLevel {
        base: 20_000,
        lapse_rate: STRATOSPHERIC_TEMPERATURE_LAPSE,
        base_temperature: -56.5,
    },
    // Stratosphere
    &AtmosphericLevel {
        base: 32_000,
        lapse_rate: 0.0,
        base_temperature: -44.5,
    },
];

///
///
/// # Arguments
///
/// * `elevation`: Elevation in meters to get the ICAO default temperature for
///
/// returns: Result<f64, UndefinedError>
///
/// # Examples
///
/// ```
/// let temp: f64 = icao_temperature_by_elevation(113.7).unwrap();
/// ```
pub fn icao_temperature_by_elevation(elevation: f64) -> Result<f64, UndefinedError> {
    if elevation < ICAO_MINIMUM_ELEVATION {
        return Err(UndefinedError(
            "The ICAO standard atmosphere below -1 kilometer (-3,280 feet) is not defined.".into(),
        ));
    }

    if elevation > ICAO_MAXIMUM_ELEVATION {
        return Err(UndefinedError(
            "The ICAO standard atmosphere above 80 kilometers (262,500 feet) is not defined."
                .into(),
        ));
    }

    let current_level = atmospheric_level_by_geopotential_altitude(elevation);

    Ok(round(
        current_level.base_temperature
            - (elevation - current_level.base as f64) * current_level.lapse_rate,
        2,
    ))
}

///
///
/// # Arguments
///
/// * `qnh`: QNH for the location given in hPa
/// * `field_elevation`: Field elevation in meters
///
/// returns: f64
///
/// # Examples
///
/// ```
/// let pressure: f64 = pressure_altitude(1021, 113.7);
/// ```
pub fn pressure_altitude(qnh: f64, field_elevation: f64) -> f64 {
    field_elevation
        + ISA_TEMPERATURE / TROPOSPHERIC_TEMPERATURE_LAPSE
        * (1.0_f64
        - (qnh / ISA_PRESSURE).powf(
        SPECIFIC_GAS_CONSTANT * TROPOSPHERIC_TEMPERATURE_LAPSE
            / GRAVITATIONAL_ACCELERATION,
    ))
}

fn atmospheric_level_by_geopotential_altitude<'a>(elevation: f64) -> &'a AtmosphericLevel {
    LEVELS.iter()
        .take_while(|level| elevation >= level.base as f64)
        .last()
        .unwrap_or(LEVELS.first().unwrap())
}

fn round(x: f64, decimals: u8) -> f64 {
    let y = 10_i32.pow(decimals.into()) as f64;
    (x * y).round() / y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_altitude_example_1() {
        let result = pressure_altitude(1021.0, 113.0);
        assert_eq!(result, 48.70703054690429);
    }

    #[test]
    fn pressure_altitude_example_2() {
        let result = pressure_altitude(1013.25, 113.0);
        assert_eq!(result, 113.0);
    }

    #[test]
    fn pressure_altitude_example_3() {
        let result = pressure_altitude(1021.0, 113.0);
        assert_eq!(result, 48.70703054690429);
    }

    #[test]
    fn isa_temperature_out_or_range_negative() {
        let result = icao_temperature_by_elevation(-1000.01);
        assert!(result.is_err());
    }

    #[test]
    fn isa_temperature_negative_1000() {
        let result = icao_temperature_by_elevation(-1000.0);
        assert_eq!(result.unwrap(), 21.5_f64);
    }

    #[test]
    fn isa_temperature_0() {
        let result = icao_temperature_by_elevation(0.0);
        assert_eq!(result.unwrap(), 15.0_f64);
    }

    #[test]
    fn isa_temperature_113() {
        let result = icao_temperature_by_elevation(113.7);
        assert_eq!(result.unwrap(), 14.26_f64);
    }

    #[test]
    fn isa_temperature_1000() {
        let result = icao_temperature_by_elevation(1000.0);
        assert_eq!(result.unwrap(), 8.5_f64);
    }

    #[test]
    fn isa_temperature_2000() {
        let result = icao_temperature_by_elevation(2000.0);
        assert_eq!(result.unwrap(), 2.0_f64);
    }

    #[test]
    fn isa_temperature_3000() {
        let result = icao_temperature_by_elevation(3000.0);
        assert_eq!(result.unwrap(), -4.5_f64);
    }

    #[test]
    fn isa_temperature_5000() {
        let result = icao_temperature_by_elevation(5000.0);
        assert_eq!(result.unwrap(), -17.5_f64);
    }

    #[test]
    fn isa_temperature_7000() {
        let result = icao_temperature_by_elevation(7000.0);
        assert_eq!(result.unwrap(), -30.5_f64);
    }

    #[test]
    fn isa_temperature_9000() {
        let result = icao_temperature_by_elevation(9_000.0);
        assert_eq!(result.unwrap(), -43.5_f64);
    }

    #[test]
    fn isa_temperature_11000() {
        let result = icao_temperature_by_elevation(11_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_13000() {
        let result = icao_temperature_by_elevation(13_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_20000() {
        let result = icao_temperature_by_elevation(20_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_25000() {
        let result = icao_temperature_by_elevation(25_000.0);
        assert_eq!(result.unwrap(), -61.5_f64);
    }

    #[test]
    fn isa_temperature_32000() {
        let result = icao_temperature_by_elevation(32_000.0);
        assert_eq!(result.unwrap(), -44.5_f64);
    }

    #[test]
    fn isa_temperature_80000() {
        let result = icao_temperature_by_elevation(80_000.0);
        assert_eq!(result.unwrap(), -44.5_f64);
    }

    #[test]
    fn isa_temperature_out_of_range_positive() {
        let result = icao_temperature_by_elevation(80_000.01_f64);
        assert!(result.is_err());
    }
}
