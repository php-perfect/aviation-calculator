use snafu::prelude::*;

// https://www.dwd.de/DE/service/lexikon/begriffe/S/Standardatmosphaere_pdf.pdf?__blob=publicationFile&v=3
const ISA_TEMPERATURE: f64 = 288.15_f64; /* K */
const ISA_PRESSURE: f64 = 1013.25_f64; /* hPa */
const TROPOSPHERIC_TEMPERATURE_LAPSE: f64 = 0.0065_f64; /* K m-1 */
const STRATOSPHERIC_TEMPERATURE_LAPSE: f64 = 0.0010_f64; /* K m-1 */
const SPECIFIC_GAS_CONSTANT: f64 = 287.058_f64;
const GRAVITATIONAL_ACCELERATION: f64 = 9.81_f64; /* m/s */
const ICAO_MINIMUM_PRESSURE_ALTITUDE: f64 = -1_000.0_f64; /* m */
const ICAO_MAXIMUM_PRESSURE_ALTITUDE: f64 = 80_000.0_f64; /* m */

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Snafu)]
pub enum UndefinedPressureAltitudeError {
    #[snafu(display("The pressure altitude {pressure_altitude} m is below the minimum defined ({min} m) in the ICAO Standard Atmosphere"))]
    BelowMinimum { min: f64, pressure_altitude: f64 },

    #[snafu(display("The pressure altitude {pressure_altitude} m is above the maximum defined ({max} m) in the ICAO Standard Atmosphere"))]
    AboveMaximum { max: f64, pressure_altitude: f64 },
}

///
///
/// # Arguments
///
/// * `pressure_altitude`: Pressure altitude in meters
///
/// returns: Result<f64, UndefinedPressureAltitudeError> Default temperature for the given pressure altitude
///
/// # Examples
///
/// ```
/// use aviation_calculator::meteorology::*;
///
/// let temp: f64 = icao_temperature(113.7).unwrap();
/// ```
pub fn icao_temperature(pressure_altitude: f64) -> Result<f64, UndefinedPressureAltitudeError> {
    if pressure_altitude < ICAO_MINIMUM_PRESSURE_ALTITUDE {
        return Err(UndefinedPressureAltitudeError::BelowMinimum { min: ICAO_MINIMUM_PRESSURE_ALTITUDE, pressure_altitude });
    }

    if pressure_altitude > ICAO_MAXIMUM_PRESSURE_ALTITUDE {
        return Err(UndefinedPressureAltitudeError::AboveMaximum { max: ICAO_MAXIMUM_PRESSURE_ALTITUDE, pressure_altitude });
    }

    let current_level = atmospheric_level_by_geopotential_altitude(pressure_altitude);

    Ok(round(current_level.base_temperature - (pressure_altitude - current_level.base as f64) * current_level.lapse_rate, 2))
}

/// # Calculate Pressure Altitude by QNH and Field Elevation
///
/// ## Arguments
///
/// * `qnh`: QNH for the location given in hPa
/// * `field_elevation`: Field elevation given in meters
///
/// returns: f64
///
/// # Examples
///
/// ```
/// use aviation_calculator::meteorology::*;
///
/// let pressure: f64 = pressure_altitude_by_qnh(1021.0, 113.7);
/// ```
pub fn pressure_altitude_by_qnh(qnh: f64, field_elevation: f64) -> f64 {
    round(field_elevation
              + ISA_TEMPERATURE / TROPOSPHERIC_TEMPERATURE_LAPSE
        * (1.0_f64
        - (qnh / ISA_PRESSURE).powf(
        SPECIFIC_GAS_CONSTANT * TROPOSPHERIC_TEMPERATURE_LAPSE
            / GRAVITATIONAL_ACCELERATION,
    )), 2)
}

///
///
/// # Arguments
///
/// * `pressure_altitude`: Pressure altitude in meters
/// * `temperature`: Current temperature in °C
///
/// returns: Result<f64, UndefinedPressureAltitudeError>
///
/// # Examples
///
/// ```
/// use aviation_calculator::meteorology::*;
///
/// let temperature_deviation = calculate_temperature_deviation(113.0, 21.0);
/// ```
pub fn calculate_temperature_deviation(pressure_altitude: f64, temperature: f64) -> Result<f64, UndefinedPressureAltitudeError> {
    Ok(round(temperature - icao_temperature(pressure_altitude)?, 2))
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
        let result = pressure_altitude_by_qnh(1021.0, 113.0);
        assert_eq!(result, 48.71);
    }

    #[test]
    fn pressure_altitude_example_2() {
        let result = pressure_altitude_by_qnh(1013.25, 113.0);
        assert_eq!(result, 113.0);
    }

    #[test]
    fn pressure_altitude_example_3() {
        let result = pressure_altitude_by_qnh(1021.0, 113.0);
        assert_eq!(result, 48.71);
    }

    #[test]
    fn isa_temperature_out_or_range_negative() {
        let result = icao_temperature(-1000.01);
        assert!(result.is_err());
    }

    #[test]
    fn isa_temperature_negative_1000() {
        let result = icao_temperature(-1000.0);
        assert_eq!(result.unwrap(), 21.5_f64);
    }

    #[test]
    fn isa_temperature_0() {
        let result = icao_temperature(0.0);
        assert_eq!(result.unwrap(), 15.0_f64);
    }

    #[test]
    fn isa_temperature_113() {
        let result = icao_temperature(113.7);
        assert_eq!(result.unwrap(), 14.26_f64);
    }

    #[test]
    fn isa_temperature_1000() {
        let result = icao_temperature(1000.0);
        assert_eq!(result.unwrap(), 8.5_f64);
    }

    #[test]
    fn isa_temperature_2000() {
        let result = icao_temperature(2000.0);
        assert_eq!(result.unwrap(), 2.0_f64);
    }

    #[test]
    fn isa_temperature_3000() {
        let result = icao_temperature(3000.0);
        assert_eq!(result.unwrap(), -4.5_f64);
    }

    #[test]
    fn isa_temperature_5000() {
        let result = icao_temperature(5000.0);
        assert_eq!(result.unwrap(), -17.5_f64);
    }

    #[test]
    fn isa_temperature_7000() {
        let result = icao_temperature(7000.0);
        assert_eq!(result.unwrap(), -30.5_f64);
    }

    #[test]
    fn isa_temperature_9000() {
        let result = icao_temperature(9_000.0);
        assert_eq!(result.unwrap(), -43.5_f64);
    }

    #[test]
    fn isa_temperature_11000() {
        let result = icao_temperature(11_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_13000() {
        let result = icao_temperature(13_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_20000() {
        let result = icao_temperature(20_000.0);
        assert_eq!(result.unwrap(), -56.5_f64);
    }

    #[test]
    fn isa_temperature_25000() {
        let result = icao_temperature(25_000.0);
        assert_eq!(result.unwrap(), -61.5_f64);
    }

    #[test]
    fn isa_temperature_32000() {
        let result = icao_temperature(32_000.0);
        assert_eq!(result.unwrap(), -44.5_f64);
    }

    #[test]
    fn isa_temperature_80000() {
        let result = icao_temperature(80_000.0);
        assert_eq!(result.unwrap(), -44.5_f64);
    }

    #[test]
    fn isa_temperature_out_of_range_positive() {
        let result = icao_temperature(80_000.01_f64);
        assert!(result.is_err());
    }

    #[test]
    fn fsm75_3_example1() {
        let result = icao_temperature(182.88);
        assert_eq!(result.unwrap(), 13.81); // 14.0
    }

    #[test]
    fn fsm75_3_example2() {
        let result = icao_temperature(609.6);
        assert_eq!(result.unwrap(), 11.04); // 11.0
    }

    #[test]
    fn fsm75_3_example3() {
        let result = icao_temperature(350.52);
        assert_eq!(result.unwrap(), 12.72); // 13.0
    }

    #[test]
    fn temperature_deviation_msl() {
        let result = calculate_temperature_deviation(0.0, 15.0);
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn temperature_deviation_positive() {
        let result = calculate_temperature_deviation(0.0, 16.0);
        assert_eq!(result.unwrap(), 1.0);
    }

    #[test]
    fn temperature_deviation_negative() {
        let result = calculate_temperature_deviation(0.0, 14.0);
        assert_eq!(result.unwrap(), -1.0);
    }

    #[test]
    fn temperature_deviation_below_msl() {
        let result = calculate_temperature_deviation(-200.0, 15.0);
        assert_eq!(result.unwrap(), -1.3);
    }

    #[test]
    fn temperature_deviation_above_msl() {
        let result = calculate_temperature_deviation(200.0, 15.0);
        assert_eq!(result.unwrap(), 1.3);
    }
}
