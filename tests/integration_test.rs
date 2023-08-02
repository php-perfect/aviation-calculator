use aviation_calculator::fk9::*;
use aviation_calculator::meteorology::pressure_altitude_by_qnh;
use aviation_calculator::utils::{feet_to_meter, meter_to_feet, round};

#[test]
fn common_takeoff_calculation_zellhausen() {
    let result: TakeoffResult = calculate_takeoff_distance(Engine::Rotax912Uls, 520.0, 370.73, 21.0, 0.0, Some(GrassSurface {
        wet: true,
        ..GrassSurface::default()
    }), SurfaceCondition::Inconspicuous);
    let takeoff_distances = result.expect("No error is expected for this takeoff calculation!");
    assert_eq!(takeoff_distances, (138.73, 344.18));
}

#[test]
fn common_takeoff_calculation_frankfurt() {
    let result: TakeoffResult = calculate_takeoff_distance(Engine::Rotax912Uls, 520.0, 364.0, 21.0, 0.0, None, SurfaceCondition::Inconspicuous);
    let takeoff_distances = result.expect("No error is expected for this takeoff calculation!");
    assert_eq!(takeoff_distances, (115.52, 286.61));
}

#[test]
fn common_pressure_altitude() {
    let result: f64 = pressure_altitude_by_qnh(996.0, 113.7);
    assert_eq!(result, 258.25);
}

#[test]
fn pressure_altitude_in_feet() {
    let result: f64 = round(meter_to_feet(pressure_altitude_by_qnh(996.0, feet_to_meter(364.0))), 1);
    assert_eq!(result, 838.2);
}

#[test]
fn pressure_altitude_in_feed_check_rounding() {
    let result: f64 = round(meter_to_feet(pressure_altitude_by_qnh(1013.25, feet_to_meter(364.0))), 1);
    assert_eq!(result, 364.0);
}
