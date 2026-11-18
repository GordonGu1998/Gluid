//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::thread;

// Import the Interface to Test=====================================================================
use gluid::{AbstractState, COOLPROP_VERSION, Output, Pair, Variable, get_fluid_json};

//==================================================================================================
// Function Name: get_nitrogen
//   Description: Create a state of nitrogen at 2 MPa and 300 K
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn get_nitrogen() -> AbstractState {
    let mut state = AbstractState::new("Nitrogen").expect("nitrogen exists");
    state
        .set_state(Pair::PressureTemperature, 2.0e6, 300.0)
        .expect("the state is valid");
    state
}

//==================================================================================================
// Function Name: version_is_the_vendored_one
//   Description: Check that the linked CoolProp is the vendored release
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn version_is_the_vendored_one() {
    assert_eq!(COOLPROP_VERSION, "8.0.0");
}

//==================================================================================================
// Function Name: nitrogen_transport_and_heat_capacity
//   Description: Check the conductivity and the heat capacity of nitrogen at 2 MPa and 300 K
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn nitrogen_transport_and_heat_capacity() {
    let mut state = get_nitrogen();
    let conductivity = state.get_output(Output::Conductivity).unwrap();
    let specific_heat = state.get_output(Output::SpecificHeat).unwrap();
    assert!(
        (conductivity / 2.670885e-2 - 1.0).abs() < 1e-6,
        "{conductivity}"
    );
    assert!((specific_heat - 1072.14).abs() < 0.01, "{specific_heat}");
}

//==================================================================================================
// Function Name: inputs_come_back_as_given
//   Description: Check that the inputs of an update are returned exactly as given
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn inputs_come_back_as_given() {
    let mut state = get_nitrogen();
    assert_eq!(state.get_output(Output::Pressure).unwrap(), 2.0e6);
    assert_eq!(state.get_output(Output::Temperature).unwrap(), 300.0);
}

//==================================================================================================
// Function Name: enthalpy_and_pressure_return_the_temperature
//   Description: Check the enthalpy and pressure flash against the temperature of the start
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn enthalpy_and_pressure_return_the_temperature() {
    let mut state = get_nitrogen();
    let enthalpy = state.get_output(Output::Enthalpy).unwrap();
    let density = state.get_output(Output::Density).unwrap();
    state
        .set_state(Pair::EnthalpyPressure, enthalpy, 2.0e6)
        .unwrap();
    let temperature = state.get_output(Output::Temperature).unwrap();
    assert!((temperature - 300.0).abs() < 1e-6, "{temperature}");
    assert!((state.get_output(Output::Density).unwrap() / density - 1.0).abs() < 1e-9);
}

//==================================================================================================
// Function Name: carbon_dioxide_saturation
//   Description: Check the saturation temperature and the critical pressure of carbon dioxide
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn carbon_dioxide_saturation() {
    let mut state = AbstractState::new("CO2").unwrap();
    state.set_state(Pair::PressureQuality, 5.0e6, 0.0).unwrap();
    let liquid = state.get_output(Output::Density).unwrap();
    let temperature = state.get_output(Output::Temperature).unwrap();
    state.set_state(Pair::PressureQuality, 5.0e6, 1.0).unwrap();
    let vapor = state.get_output(Output::Density).unwrap();
    assert!((temperature - 287.4339).abs() < 1e-3, "{temperature}");
    assert!(liquid > vapor);
    let critical = state.get_output(Output::CriticalPressure).unwrap();
    assert!((critical / 7.3773e6 - 1.0).abs() < 1e-4, "{critical}");
}

//==================================================================================================
// Function Name: clone_replays_the_last_update
//   Description: Check that a clone of a state gives the same density
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn clone_replays_the_last_update() {
    let mut state = get_nitrogen();
    let mut copy = state.clone();
    assert_eq!(
        state.get_output(Output::Density).unwrap(),
        copy.get_output(Output::Density).unwrap()
    );
}

//==================================================================================================
// Function Name: unknown_fluid_is_an_error_with_a_message
//   Description: Check that an unknown fluid is refused with a message
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn unknown_fluid_is_an_error_with_a_message() {
    let error = AbstractState::new("NoSuchFluid")
        .err()
        .expect("the fluid is refused");
    assert!(!error.is_empty());
    assert!(get_fluid_json("NoSuchFluid").is_none());
    assert!(get_fluid_json("Nitrogen").is_some_and(|text| text.contains("Nitrogen")));
}

//==================================================================================================
// Function Name: states_work_on_threads_at_once
//   Description: Check that states on eight threads give the density of a single thread
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn states_work_on_threads_at_once() {
    let expected = get_nitrogen().get_output(Output::Density).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                let mut state = get_nitrogen();
                (0..200)
                    .map(|_| state.get_output(Output::Density).unwrap())
                    .last()
                    .unwrap()
            })
        })
        .collect();
    for handle in handles {
        assert_eq!(handle.join().unwrap(), expected);
    }
}

//==================================================================================================
// Function Name: get_states
//   Description: Create single phase states of four fluids from gas to dense
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn get_states() -> Vec<AbstractState> {
    [
        ("Nitrogen", 2.0e6, 300.0),
        ("CO2", 3.0e6, 320.0),
        ("CO2", 1.2e7, 305.0),
        ("Air", 5.0e5, 400.0),
    ]
    .into_iter()
    .map(|(fluid, pressure, temperature)| {
        let mut state = AbstractState::new(fluid).expect("the fluid exists");
        state
            .set_state(Pair::PressureTemperature, pressure, temperature)
            .expect("the state is valid");
        state
    })
    .collect()
}

//==================================================================================================
// Function Name: density_derivative_matches_the_identity
//   Description: Check the density slope at constant enthalpy against the thermodynamic identity
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn density_derivative_matches_the_identity() {
    for mut state in get_states() {
        let rho = state.get_output(Output::Density).unwrap();
        let kappa = state.get_output(Output::IsothermalCompressibility).unwrap();
        let beta = state.get_output(Output::IsobaricExpansion).unwrap();
        let c_p = state.get_output(Output::SpecificHeat).unwrap();
        let t = state.get_output(Output::Temperature).unwrap();
        let identity = rho * kappa + beta * (1.0 - t * beta) / c_p;
        let derivative = state
            .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
            .unwrap();
        assert!(
            (derivative / identity - 1.0).abs() < 1e-8,
            "{derivative} {identity}"
        );
    }
}

//==================================================================================================
// Function Name: enthalpy_derivatives_match_the_closed_forms
//   Description: Check the density and temperature slopes with the enthalpy at constant pressure
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn enthalpy_derivatives_match_the_closed_forms() {
    for mut state in get_states() {
        let rho = state.get_output(Output::Density).unwrap();
        let beta = state.get_output(Output::IsobaricExpansion).unwrap();
        let c_p = state.get_output(Output::SpecificHeat).unwrap();
        let rho_h = state
            .get_derivative(Variable::Density, Variable::Enthalpy, Variable::Pressure)
            .unwrap();
        let t_h = state
            .get_derivative(
                Variable::Temperature,
                Variable::Enthalpy,
                Variable::Pressure,
            )
            .unwrap();
        assert!((rho_h / (-rho * beta / c_p) - 1.0).abs() < 1e-8, "{rho_h}");
        assert!((t_h * c_p - 1.0).abs() < 1e-8, "{t_h}");
    }
}

//==================================================================================================
// Function Name: density_derivative_matches_finite_differences
//   Description: Check both density slopes against central differences of flashed states
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn density_derivative_matches_finite_differences() {
    for mut state in get_states() {
        let p = state.get_output(Output::Pressure).unwrap();
        let h = state.get_output(Output::Enthalpy).unwrap();
        let (dp, dh) = (1.0e-4 * p, 1.0);
        let mut read = |pressure: f64, enthalpy: f64| {
            state
                .set_state(Pair::EnthalpyPressure, enthalpy, pressure)
                .unwrap();
            state.get_output(Output::Density).unwrap()
        };
        let by_pressure = (read(p + dp, h) - read(p - dp, h)) / (2.0 * dp);
        let by_enthalpy = (read(p, h + dh) - read(p, h - dh)) / (2.0 * dh);
        state.set_state(Pair::EnthalpyPressure, h, p).unwrap();
        let rho_p = state
            .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
            .unwrap();
        let rho_h = state
            .get_derivative(Variable::Density, Variable::Enthalpy, Variable::Pressure)
            .unwrap();
        assert!(
            (rho_p / by_pressure - 1.0).abs() < 1e-4,
            "{rho_p} {by_pressure}"
        );
        assert!(
            (rho_h / by_enthalpy - 1.0).abs() < 1e-4,
            "{rho_h} {by_enthalpy}"
        );
    }
}

//==================================================================================================
// Function Name: derivative_refuses_two_phase_and_repeated_variables
//   Description: Check that a two phase state and a repeated variable are refused
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn derivative_refuses_two_phase_and_repeated_variables() {
    let mut state = AbstractState::new("CO2").unwrap();
    state.set_state(Pair::PressureQuality, 5.0e6, 0.5).unwrap();
    let error = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .unwrap_err();
    assert!(error.contains("two-phase"), "{error}");

    let mut state = get_nitrogen();
    let error = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Pressure)
        .unwrap_err();
    assert!(error.contains("different"), "{error}");
}

//==================================================================================================
// Function Name: get_mixture_density
//   Description: Blend the saturated densities of a fluid at a pressure and an enthalpy
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_mixture_density(fluid: &str, pressure: f64, enthalpy: f64) -> f64 {
    let mut state = AbstractState::new(fluid).expect("the fluid exists");
    let mut get_saturated = |quality: f64| {
        state
            .set_state(Pair::PressureQuality, pressure, quality)
            .unwrap();
        (
            state.get_output(Output::Enthalpy).unwrap(),
            state.get_output(Output::Density).unwrap(),
        )
    };
    let (h_liquid, rho_liquid) = get_saturated(0.0);
    let (h_vapor, rho_vapor) = get_saturated(1.0);
    let quality = (enthalpy - h_liquid) / (h_vapor - h_liquid);
    1.0 / (quality / rho_vapor + (1.0 - quality) / rho_liquid)
}

//==================================================================================================
// Function Name: flow_state_slope_matches_density_differences
//   Description: Check the density slope in both regions against differences of the density
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
#[test]
fn flow_state_slope_matches_density_differences() {
    for (fluid, pressure) in [
        ("Water", 2.0e5),
        ("Nitrogen", 5.0e5),
        ("CO2", 5.0e6),
        ("R134a", 8.0e5),
        ("Air", 6.0e5),
    ] {
        let mut state = AbstractState::new(fluid).unwrap();
        for quality in [0.1, 0.5, 0.9] {
            state
                .set_state(Pair::PressureQuality, pressure, quality)
                .unwrap();
            let enthalpy = state.get_output(Output::Enthalpy).unwrap();
            let flow = state.get_flow_state(pressure, enthalpy, true).unwrap();
            let slope = flow.density_slope.unwrap();
            let step = 1.0e-4 * pressure;
            let reference = (get_mixture_density(fluid, pressure + step, enthalpy)
                - get_mixture_density(fluid, pressure - step, enthalpy))
                / (2.0 * step);
            assert!(
                (slope / reference - 1.0).abs() < 1e-5,
                "{fluid} {quality} {slope} {reference}"
            );
            let density = get_mixture_density(fluid, pressure, enthalpy);
            assert!(
                (flow.density / density - 1.0).abs() < 1e-12,
                "{fluid} {quality}"
            );
        }
    }
}

//==================================================================================================
// Function Name: flow_state_single_phase_matches_the_derivative
//   Description: Check the single phase state against the density and the library derivative
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
#[test]
fn flow_state_single_phase_matches_the_derivative() {
    let mut state = get_nitrogen();
    let enthalpy = state.get_output(Output::Enthalpy).unwrap();
    let density = state.get_output(Output::Density).unwrap();
    let viscosity = state.get_output(Output::Viscosity).unwrap();
    let slope = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .unwrap();
    let flow = state.get_flow_state(2.0e6, enthalpy, true).unwrap();
    assert!(
        (flow.density / density - 1.0).abs() < 1e-8,
        "{}",
        flow.density
    );
    assert!(
        (flow.viscosity / viscosity - 1.0).abs() < 1e-8,
        "{}",
        flow.viscosity
    );
    assert!((flow.density_slope.unwrap() / slope - 1.0).abs() < 1e-8);
    assert_eq!(
        state
            .get_flow_state(2.0e6, enthalpy, false)
            .unwrap()
            .density_slope,
        None
    );
}

//==================================================================================================
// Function Name: thermal_state_follows_the_mixture_rules
//   Description: Check the two phase thermal state against the reciprocal and weighted means
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
#[test]
fn thermal_state_follows_the_mixture_rules() {
    let (pressure, quality) = (5.0e6, 0.3);
    let mut state = AbstractState::new("CO2").unwrap();
    let mut get_saturated = |quality: f64| {
        state
            .set_state(Pair::PressureQuality, pressure, quality)
            .unwrap();
        [
            state.get_output(Output::Enthalpy).unwrap(),
            state.get_output(Output::Conductivity).unwrap(),
            state.get_output(Output::SpecificHeat).unwrap(),
            state.get_output(Output::Temperature).unwrap(),
        ]
    };
    let [h_liquid, k_liquid, c_liquid, _] = get_saturated(0.0);
    let [h_vapor, k_vapor, c_vapor, t_vapor] = get_saturated(1.0);
    let enthalpy = h_liquid + quality * (h_vapor - h_liquid);

    let thermal = state.get_thermal_state(pressure, enthalpy).unwrap();
    let conductivity = 1.0 / (quality / k_vapor + (1.0 - quality) / k_liquid);
    let specific_heat = quality * c_vapor + (1.0 - quality) * c_liquid;
    assert!((thermal.conductivity / conductivity - 1.0).abs() < 1e-12);
    assert!((thermal.specific_heat / specific_heat - 1.0).abs() < 1e-12);
    assert_eq!(thermal.temperature, t_vapor);
    assert_eq!(thermal.temperature_slope, 0.0);

    let mut gas = get_nitrogen();
    let enthalpy = gas.get_output(Output::Enthalpy).unwrap();
    let specific_heat = gas.get_output(Output::SpecificHeat).unwrap();
    let thermal = gas.get_thermal_state(2.0e6, enthalpy).unwrap();
    assert!((thermal.temperature_slope * specific_heat - 1.0).abs() < 1e-12);
}

//==================================================================================================
// Function Name: flow_state_works_on_a_fresh_state_and_near_the_limits
//   Description: Check a fresh state, a pressure below the triple point and one near the critical
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
#[test]
fn flow_state_works_on_a_fresh_state_and_near_the_limits() {
    let mut fresh = AbstractState::new("Nitrogen").unwrap();
    assert!(fresh.get_flow_state(2.0e6, 4.0e5, true).is_ok());

    for (fluid, ratio) in [
        ("CO2", 0.04),
        ("CO2", 0.999),
        ("Air", 0.999),
        ("Water", 0.999),
    ] {
        let mut state = AbstractState::new(fluid).unwrap();
        let pressure = ratio * state.get_output(Output::CriticalPressure).unwrap();
        state
            .set_state(Pair::PressureQuality, pressure, 0.5)
            .unwrap();
        let enthalpy = state.get_output(Output::Enthalpy).unwrap();
        let flow = state.get_flow_state(pressure, enthalpy, true).unwrap();
        let slope = flow.density_slope.unwrap();
        assert!(slope.is_finite() && slope > 0.0, "{fluid} {ratio} {slope}");
    }
}

//==================================================================================================
// Function Name: flow_state_puts_the_saturation_lines_in_the_two_phase_region
//   Description: Check that the saturated liquid and vapor take the two phase slope
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
#[test]
fn flow_state_puts_the_saturation_lines_in_the_two_phase_region() {
    let pressure = 5.0e6;
    let mut state = AbstractState::new("CO2").unwrap();
    state
        .set_state(Pair::PressureQuality, pressure, 0.0)
        .unwrap();
    let h_liquid = state.get_output(Output::Enthalpy).unwrap();
    state
        .set_state(Pair::PressureQuality, pressure, 1.0)
        .unwrap();
    let h_vapor = state.get_output(Output::Enthalpy).unwrap();

    let mut get_slope = |enthalpy: f64| {
        let flow = state.get_flow_state(pressure, enthalpy, true).unwrap();
        flow.density_slope.unwrap()
    };
    assert!(get_slope(h_liquid) > 10.0 * get_slope(h_liquid - 1.0));
    assert!(get_slope(h_vapor) > 1.0e-6 && get_slope(h_vapor + 1.0) > 0.0);
    assert!((get_slope(h_vapor) / get_slope(h_vapor + 1.0) - 1.0).abs() > 1e-3);
}

//==================================================================================================
// Function Name: prandtl_number_matches_its_definition
//   Description: Check the Prandtl number against the heat capacity viscosity and conductivity
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn prandtl_number_matches_its_definition() {
    for mut state in get_states() {
        let c_p = state.get_output(Output::SpecificHeat).unwrap();
        let mu = state.get_output(Output::Viscosity).unwrap();
        let k = state.get_output(Output::Conductivity).unwrap();
        let prandtl = state.get_output(Output::Prandtl).unwrap();
        assert!((prandtl / (c_p * mu / k) - 1.0).abs() < 1e-10, "{prandtl}");
    }
}

//==================================================================================================
// Function Name: carbon_dioxide_fixed_points
//   Description: Check the critical and triple points of carbon dioxide
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn carbon_dioxide_fixed_points() {
    let mut state = AbstractState::new("CO2").unwrap();
    for (output, expected) in [
        (Output::CriticalTemperature, 304.1282),
        (Output::TripleTemperature, 216.592),
        (Output::MinTemperature, 216.592),
        (Output::TriplePressure, 517_964.0),
    ] {
        let value = state.get_output(output).unwrap();
        assert!((value / expected - 1.0).abs() < 1e-5, "{output:?} {value}");
    }
}

//==================================================================================================
// Function Name: derivative_follows_an_enthalpy_flash
//   Description: Check that a derivative after an enthalpy and pressure update matches the start
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
#[test]
fn derivative_follows_an_enthalpy_flash() {
    let mut state = get_nitrogen();
    let expected = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .unwrap();
    let enthalpy = state.get_output(Output::Enthalpy).unwrap();
    state
        .set_state(Pair::EnthalpyPressure, enthalpy, 2.0e6)
        .unwrap();
    let derivative = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .unwrap();
    assert!((derivative / expected - 1.0).abs() < 1e-8, "{derivative}");
}
