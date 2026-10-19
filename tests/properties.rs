//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::thread;

// Import the Interface to Test=====================================================================
use gluid::{AbstractState, COOLPROP_VERSION, Output, Pair, get_fluid_json};

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
