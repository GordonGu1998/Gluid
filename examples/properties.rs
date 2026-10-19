//==================================================================================================
// Import
//==================================================================================================
// Import the Interface of the Crate================================================================
use gluid::{AbstractState, Output, Pair};

//==================================================================================================
// Function Name: main
//   Description: Print nitrogen at 2 MPa and 300 K and carbon dioxide on the saturation line
//     Edit Date: Oct 19th, 2026.
//==================================================================================================
fn main() {
    // Read Nitrogen from Pressure and Temperature==================================================
    let mut nitrogen = AbstractState::new("Nitrogen").expect("nitrogen exists");
    nitrogen
        .set_state(Pair::PressureTemperature, 2.0e6, 300.0)
        .expect("the state is valid");
    println!("Nitrogen at 2 MPa and 300 K");
    for (name, output) in [
        ("density in kg/m3", Output::Density),
        ("viscosity in Pa s", Output::Viscosity),
        ("conductivity in W/m/K", Output::Conductivity),
        ("specific heat in J/kg/K", Output::SpecificHeat),
    ] {
        let value = nitrogen.get_output(output).expect("the output is valid");
        println!("  {name}: {value:.6e}");
    }

    // Read Carbon Dioxide on the Saturation Line at 5 MPa==========================================
    let mut carbon_dioxide = AbstractState::new("CO2").expect("carbon dioxide exists");
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
}
