use std::f64::consts::PI;

pub mod fk9;
pub mod meteorology;

const FEET: f64 = 0.3048_f64; /* m */

/// # Calculate Ground Speed (GS)
///
/// course Course
///
/// tas True Air Speed (TAS)
///
/// wd Wind Direction (WD)
///
/// ws Wind Speed (WS)
pub fn calculate_ground_speed(course: f64, tas: f64, wd: f64, ws: f64) -> f64 {
    if ws == 0.0 {
        return tas;
    }

    let crs = to_radian(normalize_degree(course));
    let wind_dir = to_radian(normalize_degree(wd));
    let swc = (ws / tas) * (wind_dir - crs).sin();

    tas * (1.0 - swc.powi(2)).sqrt() - (ws * (wind_dir - crs).cos())
}

/// # Calculate Wind Correction Angle (WCA)
///
/// tas True Air Speed (TAS)
///
/// ws Wind Speed (WS)
///
/// awa Acute Wind Angle (AWA)
///
/// Wind Correction Angle (WCA)
pub fn calculate_wca(tas: f64, ws: f64, awa: f64) -> f64 {
    if awa == 0.0 || awa == 180.0 {
        return 0.0;
    }

    to_degree((ws / tas * to_radian(normalize_degree(awa)).sin()).asin())
}

/// # Calculate Heading
///
/// dc Desired Course (DC)
///
/// tas True Air Speed (TAS)
///
/// wd Wind Direction (WD)
///
/// ws Wind Speed (WS)
///
/// @return Heading
pub fn calculate_heading(dc: f64, tas: f64, wd: f64, ws: f64) -> f64 {
    dc + calculate_wca(tas, ws, wd - dc)
}

fn meter_to_feet(meter: f64) -> f64 {
    meter / FEET
}

fn feet_to_meter(meter: f64) -> f64 {
    meter * FEET
}

fn to_degree(value: f64) -> f64 {
    180.0_f64 / PI * value
}

fn to_radian(value: f64) -> f64 {
    PI / 180.0_f64 * value
}

fn normalize_degree(value: f64) -> f64 {
    value % 360.0_f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_ground_speed_no_wind() {
        let result = calculate_ground_speed(70.0, 100.0, 0.0, 0.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn calculate_ground_speed_headwind() {
        let result = calculate_ground_speed(0.0, 100.0, 0.0, 30.0);
        assert_eq!(result, 70.0);
    }

    #[test]
    fn calculate_ground_speed_tailwind() {
        let result = calculate_ground_speed(0.0, 100.0, 180.0, 20.0);
        assert_eq!(result, 120.0);
    }

    #[test]
    fn calculate_ground_speed_crosswind() {
        let result = calculate_ground_speed(180.0, 100.0, 90.0, 10.0);
        assert_eq!(result, 99.498743710662);
    }

    #[test]
    fn calculate_ground_speed_precision() {
        let result = calculate_ground_speed(45.0, 90.0, 90.0, 12.0);
        assert_eq!(result, 81.1138257641699);
    }

    #[test]
    fn calculate_wca_0() {
        let result = calculate_wca(100.0, 20.0, 90.0);
        assert_eq!(result, 11.536959032815489);
    }

    #[test]
    fn calculate_wca_1() {
        let result = calculate_wca(100.0, 20.0, 450.0);
        assert_eq!(result, 11.536959032815489);
    }

    #[test]
    fn calculate_wca_2() {
        let result = calculate_wca(100.0, 20.0, 270.0);
        assert_eq!(result, -11.536959032815489);
    }

    #[test]
    fn calculate_wca_3() {
        let result = calculate_wca(90.0, 20.0, 60.0);
        assert_eq!(result, 11.09580328313639);
    }

    #[test]
    fn calculate_wca_4() {
        let result = calculate_wca(90.0, 0.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_5() {
        let result = calculate_wca(90.0, 0.0, 360.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_6() {
        let result = calculate_wca(90.0, 0.0, 40.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_7() {
        let result = calculate_wca(90.0, 40.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_8() {
        let result = calculate_wca(90.0, 40.0, 180.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_9() {
        let result = calculate_wca(90.0, 20.0, -45.0);
        assert_eq!(result, -9.040631036927891);
    }

    #[test]
    fn calculate_wca_10() {
        let result = calculate_wca(90.0, 20.0, 100.0);
        assert_eq!(result, 12.641271896674168);
    }

    #[test]
    fn calculate_wca_11() {
        let result = calculate_wca(95.0, 11.2, 360.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_wca_12() {
        let result = calculate_wca(95.0, 11.2, -160.0);
        assert_eq!(result, -2.3109284049894776);
    }

    #[test]
    fn calculate_heading_0() {
        let result = calculate_heading(90.0, 100.0, 180.0, 20.0);
        assert_eq!(result, 101.53695903281549);
    }

    #[test]
    fn calculate_heading_1() {
        let result = calculate_heading(0.0, 100.0, 90.0, 20.0);
        assert_eq!(result, 11.536959032815489);
    }

    #[test]
    fn calculate_heading_2() {
        let result = calculate_heading(320.0, 100.0, 90.0, 23.0);
        assert_eq!(result, 330.1479291050075);
    }

    #[test]
    fn calculate_heading_3() {
        let result = calculate_heading(120.0, 90.0, 70.0, 30.0);
        assert_eq!(result, 105.20578448936156);
    }

    #[test]
    fn calculate_heading_4() {
        let result = calculate_heading(350.0, 95.0, 190.0, 10.1);
        assert_eq!(result, 347.91614336837915);
    }

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
}
