//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::thread;

// Import the Interface of the Crate================================================================
use gluid::{AbstractState, COOLPROP_VERSION, Output, Pair, Variable, get_fluid_json};

//==================================================================================================
// Function Name: get_state
//   Description: Create the state of a fluid from pressure and temperature
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn get_state(fluid: &str, pressure: f64, temperature: f64) -> AbstractState {
    let mut state = AbstractState::new(fluid).expect("the fluid exists");
    state
        .set_state(Pair::PressureTemperature, pressure, temperature)
        .expect("the state is valid");
    state
}

//==================================================================================================
// Function Name: print_transport
//   Description: Print nitrogen at 2 MPa and 300 K with the Prandtl number
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_transport() {
    let mut nitrogen = get_state("Nitrogen", 2.0e6, 300.0);
    println!("Nitrogen at 2 MPa and 300 K");
    for (name, output) in [
        ("density in kg/m3", Output::Density),
        ("enthalpy in J/kg", Output::Enthalpy),
        ("specific heat in J/kg/K", Output::SpecificHeat),
        ("viscosity in Pa s", Output::Viscosity),
        ("conductivity in W/m/K", Output::Conductivity),
        ("Prandtl number", Output::Prandtl),
        (
            "isothermal compressibility in 1/Pa",
            Output::IsothermalCompressibility,
        ),
        ("isobaric expansion in 1/K", Output::IsobaricExpansion),
    ] {
        let value = nitrogen.get_output(output).expect("the output is valid");
        println!("  {name}: {value:.6e}");
    }
}

//==================================================================================================
// Function Name: print_saturation
//   Description: Print carbon dioxide on the saturation line by pressure and by temperature
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_saturation() {
    let mut carbon_dioxide = AbstractState::new("CO2").expect("carbon dioxide exists");

    // Read the Saturated Liquid and Vapor at 5 MPa=================================================
    println!("Carbon dioxide at 5 MPa");
    for (name, quality) in [("liquid", 0.0), ("vapor", 1.0)] {
        carbon_dioxide
            .set_state(Pair::PressureQuality, 5.0e6, quality)
            .expect("the state is valid");
        let temperature = carbon_dioxide
            .get_output(Output::Temperature)
            .expect("valid");
        let density = carbon_dioxide.get_output(Output::Density).expect("valid");
        println!("  {name}: {temperature:.4} K, {density:.3} kg/m3");
    }

    // Read the Saturation Pressure at 280 K from Quality and Temperature===========================
    carbon_dioxide
        .set_state(Pair::QualityTemperature, 0.0, 280.0)
        .expect("the state is valid");
    let pressure = carbon_dioxide.get_output(Output::Pressure).expect("valid");
    println!(
        "Carbon dioxide at 280 K saturates at {:.4} MPa",
        pressure / 1.0e6
    );
}

//==================================================================================================
// Function Name: print_enthalpy_flash
//   Description: Print the temperature that an enthalpy and pressure flash recovers
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_enthalpy_flash() {
    let mut state = get_state("Nitrogen", 2.0e6, 300.0);
    let enthalpy = state.get_output(Output::Enthalpy).expect("valid");
    state
        .set_state(Pair::EnthalpyPressure, enthalpy, 2.0e6)
        .expect("the state is valid");
    let temperature = state.get_output(Output::Temperature).expect("valid");
    let pressure = state.get_output(Output::Pressure).expect("valid");
    println!("Enthalpy {enthalpy:.3} J/kg at {pressure} Pa flashes back to {temperature:.6} K");
}

//==================================================================================================
// Function Name: print_derivatives
//   Description: Print first partial derivatives and compare one with its thermodynamic identity
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_derivatives() {
    let mut state = get_state("CO2", 1.2e7, 305.0);
    println!("Carbon dioxide at 12 MPa and 305 K");

    // Read Three Derivatives of a Single Phase State===============================================
    let rho_p = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .expect("a single phase state");
    let rho_h = state
        .get_derivative(Variable::Density, Variable::Enthalpy, Variable::Pressure)
        .expect("a single phase state");
    let t_h = state
        .get_derivative(
            Variable::Temperature,
            Variable::Enthalpy,
            Variable::Pressure,
        )
        .expect("a single phase state");
    println!("  d(density)/d(pressure) at constant enthalpy: {rho_p:.6e} s2/m2");
    println!("  d(density)/d(enthalpy) at constant pressure: {rho_h:.6e} kg s2/m5");
    println!("  d(temperature)/d(enthalpy) at constant pressure: {t_h:.6e} kg K/J");

    // Compare the First One with the Identity of the Outputs=======================================
    let density = state.get_output(Output::Density).expect("valid");
    let kappa = state
        .get_output(Output::IsothermalCompressibility)
        .expect("valid");
    let beta = state.get_output(Output::IsobaricExpansion).expect("valid");
    let specific_heat = state.get_output(Output::SpecificHeat).expect("valid");
    let temperature = state.get_output(Output::Temperature).expect("valid");
    let identity = density * kappa + beta * (1.0 - temperature * beta) / specific_heat;
    println!(
        "  identity of the outputs: {identity:.6e} (ratio {:.12})",
        rho_p / identity
    );

    // Show the Two Refusals========================================================================
    let error = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Pressure)
        .expect_err("a repeated variable is refused");
    println!("  repeated variable: {error}");
    state
        .set_state(Pair::PressureQuality, 5.0e6, 0.5)
        .expect("the state is valid");
    let error = state
        .get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)
        .expect_err("a two-phase state is refused");
    println!("  two-phase state: {error}");
}

//==================================================================================================
// Function Name: print_constants
//   Description: Print the critical and triple points of carbon dioxide
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_constants() {
    let mut state = AbstractState::new("CO2").expect("carbon dioxide exists");
    println!("Carbon dioxide fixed points");
    for (name, output) in [
        ("critical temperature in K", Output::CriticalTemperature),
        ("critical pressure in Pa", Output::CriticalPressure),
        ("triple temperature in K", Output::TripleTemperature),
        ("triple pressure in Pa", Output::TriplePressure),
        (
            "lowest temperature of the equation in K",
            Output::MinTemperature,
        ),
    ] {
        let value = state.get_output(output).expect("the output is valid");
        println!("  {name}: {value:.4}");
    }
}

//==================================================================================================
// Function Name: print_errors
//   Description: Print the messages of CoolProp for an unknown fluid and an impossible state
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_errors() {
    println!("Errors carry the message of CoolProp");
    let error = AbstractState::new("NoSuchFluid")
        .err()
        .expect("the fluid is refused");
    println!("  unknown fluid: {error}");
    let mut state = AbstractState::new("Nitrogen").expect("nitrogen exists");
    let error = state
        .set_state(Pair::PressureTemperature, 1.0e6, 5.0)
        .expect_err("a temperature below the melting line is refused");
    println!("  temperature below the melting line: {error}");
    let error = state
        .set_state(Pair::PressureQuality, 1.0e6, 1.5)
        .expect_err("a quality above one is refused");
    println!("  quality above one: {error}");
}

//==================================================================================================
// Function Name: print_threads
//   Description: Print the density that four threads read from copies of one state
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn print_threads() {
    let state = get_state("Methane", 1.0e6, 300.0);
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let mut copy = state.clone();
            thread::spawn(move || copy.get_output(Output::Density).expect("valid"))
        })
        .collect();
    println!("Methane at 1 MPa and 300 K on four threads");
    for (index, handle) in handles.into_iter().enumerate() {
        let density = handle.join().expect("the thread finished");
        println!("  thread {index}: {density:.12} kg/m3");
    }
}

//==================================================================================================
// Function Name: main
//   Description: Print every feature of the crate for pure fluids
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn main() {
    println!("CoolProp {COOLPROP_VERSION} linked statically");
    print_transport();
    print_saturation();
    print_enthalpy_flash();
    print_derivatives();
    print_constants();
    print_errors();
    print_threads();

    // Read the JSON Description of a Fluid=========================================================
    let json = get_fluid_json("Nitrogen").expect("nitrogen exists");
    println!("The JSON description of nitrogen has {} bytes", json.len());
}
