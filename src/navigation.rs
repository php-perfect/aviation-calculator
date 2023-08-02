use crate::utils::*;

/// # Calculate Ground Speed (GS)
///
/// ## Arguments
///
/// * `course`: Course in degrees
/// * `tas`: True Air Speed (TAS) in any unit
/// * `wd`: Wind Direction (WD) in degrees
/// * `ws`: Wind Speed (WS) in the same unit as tas
///
/// returns: f64 GS in the same unit as TAS is provided
///
/// # Examples
///
/// ```
/// use aviation_calculator::navigation::*;
///
/// let gs = ground_speed(140.0, 110.0, 90.0, 12.0);
/// ```
pub fn ground_speed(course: f64, tas: f64, wd: f64, ws: f64) -> f64 {
    if ws == 0.0 {
        return tas;
    }

    let crs = to_radian(normalize_degree(course));
    let wind_dir = to_radian(normalize_degree(wd));
    let swc = (ws / tas) * (wind_dir - crs).sin();

    round(tas * (1.0 - swc.powi(2)).sqrt() - (ws * (wind_dir - crs).cos()), 2)
}

/// # Calculate Wind Correction Angle (WCA)
///
/// ## Arguments
///
/// * `tas`: True Air Speed (TAS) in any unit
/// * `ws`: Wind Speed (WS) in the same unit as tas
/// * `awa`: Acute Wind Angle (AWA) in degrees
///
/// returns: f64 Wind Correction Angle (WCA) in degrees
///
/// # Examples
///
/// ```
/// use aviation_calculator::navigation::*;
///
/// let wca = wind_correction_angle(110.0, 12.0, 20.0);
/// ```
pub fn wind_correction_angle(tas: f64, ws: f64, awa: f64) -> f64 {
    if awa == 0.0 || awa == 180.0 || ws == 0.0 {
        return 0.0;
    }

    round(to_degree((ws / tas * to_radian(normalize_degree(awa)).sin()).asin()), 2)
}

/// # Calculate Heading
///
/// ## Arguments
///
/// * `dc`: Desired Course (DC)
/// * `tas`: True Air Speed (TAS)
/// * `wd`: Wind Direction (WD)
/// * `ws`: Wind Speed (WS)
///
/// returns: f64 Heading
///
/// # Examples
///
/// ```
/// use aviation_calculator::navigation::*;
///
/// let heading = heading(90.0, 110.0, 180.0, 12.5);
/// ```
pub fn heading(dc: f64, tas: f64, wd: f64, ws: f64) -> f64 {
    round(dc + wind_correction_angle(tas, ws, wd - dc), 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_ground_speed_no_wind() {
        let result = ground_speed(70.0, 100.0, 0.0, 0.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn calculate_ground_speed_headwind() {
        let result = ground_speed(0.0, 100.0, 0.0, 30.0);
        assert_eq!(result, 70.0);
    }

    #[test]
    fn calculate_ground_speed_tailwind() {
        let result = ground_speed(0.0, 100.0, 180.0, 20.0);
        assert_eq!(result, 120.0);
    }

    #[test]
    fn calculate_ground_speed_crosswind() {
        let result = ground_speed(180.0, 100.0, 90.0, 10.0);
        assert_eq!(result, 99.5);
    }

    #[test]
    fn calculate_ground_speed_precision() {
        let result = ground_speed(45.0, 90.0, 90.0, 12.0);
        assert_eq!(result, 81.11);
    }

    #[test]
    fn calculate_wca_0() {
        let result = wind_correction_angle(100.0, 20.0, 90.0);
        assert_eq!(result, 11.54);
    }

    #[test]
    fn calculate_wca_1() {
        let result = wind_correction_angle(100.0, 20.0, 450.0);
        assert_eq!(result, 11.54);
    }

    #[test]
    fn calculate_wca_2() {
        let result = wind_correction_angle(100.0, 20.0, 270.0);
        assert_eq!(result, -11.54);
    }

    #[test]
    fn calculate_wca_3() {
        let result = wind_correction_angle(90.0, 20.0, 60.0);
        assert_eq!(result, 11.1);
    }

    #[test]
    fn calculate_wca_4() {
        let result = wind_correction_angle(90.0, 0.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_5() {
        let result = wind_correction_angle(90.0, 0.0, 360.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_6() {
        let result = wind_correction_angle(90.0, 0.0, 40.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_7() {
        let result = wind_correction_angle(90.0, 40.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_8() {
        let result = wind_correction_angle(90.0, 40.0, 180.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_9() {
        let result = wind_correction_angle(90.0, 20.0, -45.0);
        assert_eq!(result, -9.04);
    }

    #[test]
    fn calculate_wca_10() {
        let result = wind_correction_angle(90.0, 20.0, 100.0);
        assert_eq!(result, 12.64);
    }

    #[test]
    fn calculate_wca_11() {
        let result = wind_correction_angle(95.0, 11.2, 360.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_12() {
        let result = wind_correction_angle(95.0, 11.2, -160.0);
        assert_eq!(result, -2.31);
    }

    #[test]
    fn calculate_heading_0() {
        let result = heading(90.0, 100.0, 180.0, 20.0);
        assert_eq!(result, 101.54);
    }

    #[test]
    fn calculate_heading_1() {
        let result = heading(0.0, 100.0, 90.0, 20.0);
        assert_eq!(result, 11.54);
    }

    #[test]
    fn calculate_heading_2() {
        let result = heading(320.0, 100.0, 90.0, 23.0);
        assert_eq!(result, 330.15);
    }

    #[test]
    fn calculate_heading_3() {
        let result = heading(120.0, 90.0, 70.0, 30.0);
        assert_eq!(result, 105.21);
    }

    #[test]
    fn calculate_heading_4() {
        let result = heading(350.0, 95.0, 190.0, 10.1);
        assert_eq!(result, 347.92);
    }
}
