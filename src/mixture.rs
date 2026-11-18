//==================================================================================================
// Import
//==================================================================================================
// Import the State and Its Enumerations from the Crate Root========================================
use crate::{AbstractState, Output, Pair, Variable};

//==================================================================================================
// Constants
//==================================================================================================
// Define the Void Fraction Where the Mixture Viscosity Model Changes Its Branch====================
const VOID_FRACTION_SWITCH: f64 = 0.3;

// Define the Relative Pressure Step of the Saturation Slopes of a Fluid with a Glide===============
const SLOPE_STEP_RATIO: f64 = 1.0e-5;

// Define the Pressure Ratio of the Glide Test to the Critical Pressure=============================
const GLIDE_PRESSURE_RATIO: f64 = 0.5;

// Define the Relative Temperature Gap Above Which a Fluid Has a Glide==============================
const GLIDE_TOLERANCE: f64 = 1.0e-6;

//==================================================================================================
// Types
//==================================================================================================
// Declare the Flow Properties of a Film Fluid at a Pressure and an Enthalpy========================
/// Density, viscosity and density slope of a film fluid at a pressure and a static enthalpy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowState {
    // Define the Density in Kilogram per Cubic Meter===============================================
    pub density: f64,

    // Define the Dynamic Viscosity in Pascal Second================================================
    pub viscosity: f64,

    // Define the Density Slope in Second Squared per Meter Squared None If Not Requested===========
    pub density_slope: Option<f64>,
}

// Declare the Thermal Properties of a Film Fluid at a Pressure and an Enthalpy=====================
/// Conductivity, specific heat and temperature of a film fluid at a pressure and a static
/// enthalpy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThermalState {
    // Define the Thermal Conductivity in Watt per Meter per Kelvin=================================
    pub conductivity: f64,

    // Define the Specific Heat Capacity at Constant Pressure in Joule per Kilogram per Kelvin======
    pub specific_heat: f64,

    // Define the Temperature in Kelvin=============================================================
    pub temperature: f64,

    // Define the Temperature Slope with the Enthalpy in Kelvin Kilogram per Joule==================
    pub temperature_slope: f64,
}

// Declare the Saturated Liquid and Vapor Properties at a Pressure==================================
#[derive(Clone, Copy)]
struct Saturation {
    h_liquid: f64,
    h_vapor: f64,
    rho_liquid: f64,
    rho_vapor: f64,
}

// Declare the Region of a State from Its Pressure and Enthalpy=====================================
enum Region {
    SinglePhase,
    TwoPhase(Saturation),
}

//==================================================================================================
// Mixture Rules
//==================================================================================================
//==================================================================================================
// Function Name: get_quality
//   Description: Calculate the mixture quality from the enthalpies per Arauz and San Andres
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_quality(h: f64, h_liquid: f64, h_vapor: f64) -> f64 {
    // Interpolate the Enthalpy Linearly between the Saturated Liquid and the Saturated Vapor=======
    (h - h_liquid) / (h_vapor - h_liquid)
}

//==================================================================================================
// Function Name: get_mixture_density
//   Description: Calculate the mixture density from the quality per Arauz and San Andres
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_mixture_density(quality: f64, rho_liquid: f64, rho_vapor: f64) -> f64 {
    // Add the Specific Volumes of the Two Phases Weighted by the Mass Fractions====================
    1.0 / (quality / rho_vapor + (1.0 - quality) / rho_liquid)
}

//==================================================================================================
// Function Name: get_void_fraction
//   Description: Calculate the void fraction from the quality per Arauz and San Andres
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_void_fraction(quality: f64, rho_liquid: f64, rho_vapor: f64) -> f64 {
    // Convert the Mass Fraction of the Vapor into Its Volume Fraction==============================
    quality / (quality + (1.0 - quality) * rho_vapor / rho_liquid)
}

//==================================================================================================
// Function Name: get_mixture_conductivity
//   Description: Calculate the mixture thermal conductivity with the reciprocal law in the quality
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_mixture_conductivity(quality: f64, k_liquid: f64, k_vapor: f64) -> f64 {
    // Add the Reciprocal Conductivities of the Two Phases Weighted by the Mass Fractions===========
    1.0 / (quality / k_vapor + (1.0 - quality) / k_liquid)
}

//==================================================================================================
// Function Name: get_mixture_specific_heat
//   Description: Calculate the mixture specific heat as the quality weighted saturated mean
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_mixture_specific_heat(quality: f64, c_liquid: f64, c_vapor: f64) -> f64 {
    quality * c_vapor + (1.0 - quality) * c_liquid
}

//==================================================================================================
// Function Name: get_mixture_viscosity
//   Description: Calculate the mixture viscosity per Arauz and San Andres
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_mixture_viscosity(
    quality: f64,
    void_fraction: f64,
    rho_liquid: f64,
    rho_vapor: f64,
    mu_liquid: f64,
    mu_vapor: f64,
) -> f64 {
    if void_fraction <= VOID_FRACTION_SWITCH {
        // Apply the Dilute Suspension Model for Void Fractions up to Three Tenths==================
        let ratio = mu_vapor / mu_liquid;

        mu_liquid * (1.0 + 2.5 * (ratio + 0.4) / (ratio + 1.0) * void_fraction)
    } else {
        // Calculate the Quality and the Viscosity at the Void Fraction of Three Tenths=============
        let quality_plus = VOID_FRACTION_SWITCH
            / (VOID_FRACTION_SWITCH + (1.0 - VOID_FRACTION_SWITCH) * rho_liquid / rho_vapor);
        let mu_plus =
            (1.3 * mu_liquid * mu_liquid + 1.75 * mu_liquid * mu_vapor) / (mu_liquid + mu_vapor);
        let inv_mu_plus = 1.0 / mu_plus;

        // Interpolate the Reciprocal Viscosity between Three Tenths Void Fraction and the Vapor====
        (quality_plus - 1.0)
            / ((quality_plus / mu_vapor - inv_mu_plus) + quality * (inv_mu_plus - 1.0 / mu_vapor))
    }
}

//==================================================================================================
// Film Fluid Properties
//==================================================================================================
impl AbstractState {
    //==============================================================================================
    // Function Name: get_flow_state
    //   Description: Calculate the density viscosity and on request the density slope
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Read the density, the viscosity and, if `with_slope` is set, the slope of the density with
    /// the pressure at constant enthalpy, all in SI units, at a pressure and a static enthalpy.
    ///
    /// A single-phase state returns the CoolProp properties. A two-phase state is the homogeneous
    /// mixture of the saturated liquid and vapor: the quality follows from the enthalpy, the
    /// density from the specific volumes, the viscosity from the model of Arauz and San Andres.
    /// The slope of a single-phase state is CoolProp's derivative at constant enthalpy. In the
    /// two-phase region it is CoolProp's two-phase derivative, except for a fluid with a
    /// temperature glide (air), where CoolProp's formula does not match its own saturated
    /// properties: there the slope follows from central differences of those properties. The
    /// saturation lines belong to the two-phase region (quality 0 and 1), and the slope jumps
    /// across them. The call changes the current state.
    pub fn get_flow_state(
        &mut self,
        pressure: f64,
        enthalpy: f64,
        with_slope: bool,
    ) -> Result<FlowState, String> {
        match self.get_region(pressure, enthalpy)? {
            Region::SinglePhase => {
                // Flash the State Once and Read Only the Requested Properties======================
                self.set_state(Pair::EnthalpyPressure, enthalpy, pressure)?;
                let density = self.get_output(Output::Density)?;
                let viscosity = self.get_output(Output::Viscosity)?;
                let density_slope = if with_slope {
                    Some(self.get_derivative(
                        Variable::Density,
                        Variable::Pressure,
                        Variable::Enthalpy,
                    )?)
                } else {
                    None
                };

                Ok(FlowState {
                    density,
                    viscosity,
                    density_slope,
                })
            }

            Region::TwoPhase(saturation) => {
                // Blend the Densities and Calculate the Void Fraction==============================
                let quality = get_quality(enthalpy, saturation.h_liquid, saturation.h_vapor);
                let density =
                    get_mixture_density(quality, saturation.rho_liquid, saturation.rho_vapor);
                let void_fraction =
                    get_void_fraction(quality, saturation.rho_liquid, saturation.rho_vapor);

                // Read the Saturated Viscosities Only for Two Phase States=========================
                self.set_state(Pair::PressureQuality, pressure, 1.0)?;
                let mu_vapor = self.get_output(Output::Viscosity)?;
                self.set_state(Pair::PressureQuality, pressure, 0.0)?;
                let mu_liquid = self.get_output(Output::Viscosity)?;

                // Blend the Viscosities============================================================
                let viscosity = get_mixture_viscosity(
                    quality,
                    void_fraction,
                    saturation.rho_liquid,
                    saturation.rho_vapor,
                    mu_liquid,
                    mu_vapor,
                );

                // Read the Density Slope at Constant Enthalpy on Request===========================
                let density_slope = if with_slope {
                    Some(self.get_two_phase_slope_of(pressure, quality, &saturation)?)
                } else {
                    None
                };

                Ok(FlowState {
                    density,
                    viscosity,
                    density_slope,
                })
            }
        }
    }

    //==============================================================================================
    // Function Name: get_thermal_state
    //   Description: Calculate the thermal conductivity specific heat and temperature of a state
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Read the thermal conductivity, the specific heat, the temperature and the slope of the
    /// temperature with the enthalpy, all in SI units, at a pressure and a static enthalpy.
    ///
    /// A single-phase state returns the CoolProp properties and the slope `1 / c_p`. A
    /// two-phase state is the homogeneous mixture of the saturated liquid and vapor: the
    /// conductivity follows the reciprocal law in the quality, the specific heat the quality
    /// weighted mean without latent heat, the temperature is that of the saturated vapor, and the
    /// slope is zero. The call changes the current state.
    pub fn get_thermal_state(
        &mut self,
        pressure: f64,
        enthalpy: f64,
    ) -> Result<ThermalState, String> {
        match self.get_region(pressure, enthalpy)? {
            Region::SinglePhase => {
                // Flash the State Once and Read the Three Properties===============================
                self.set_state(Pair::EnthalpyPressure, enthalpy, pressure)?;

                let specific_heat = self.get_output(Output::SpecificHeat)?;

                Ok(ThermalState {
                    conductivity: self.get_output(Output::Conductivity)?,
                    specific_heat,
                    temperature: self.get_output(Output::Temperature)?,
                    temperature_slope: 1.0 / specific_heat,
                })
            }

            Region::TwoPhase(saturation) => {
                let quality = get_quality(enthalpy, saturation.h_liquid, saturation.h_vapor);

                // Read the Saturated Vapor Which Also Gives the Saturation Temperature=============
                self.set_state(Pair::PressureQuality, pressure, 1.0)?;
                let k_vapor = self.get_output(Output::Conductivity)?;
                let c_vapor = self.get_output(Output::SpecificHeat)?;
                let temperature = self.get_output(Output::Temperature)?;

                // Read the Saturated Liquid========================================================
                self.set_state(Pair::PressureQuality, pressure, 0.0)?;
                let k_liquid = self.get_output(Output::Conductivity)?;
                let c_liquid = self.get_output(Output::SpecificHeat)?;

                Ok(ThermalState {
                    conductivity: get_mixture_conductivity(quality, k_liquid, k_vapor),
                    specific_heat: get_mixture_specific_heat(quality, c_liquid, c_vapor),
                    temperature,
                    temperature_slope: 0.0,
                })
            }
        }
    }

    //==============================================================================================
    // Function Name: get_region
    //   Description: Classify a state as single phase or two phase from its pressure and enthalpy
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_region(&mut self, pressure: f64, enthalpy: f64) -> Result<Region, String> {
        // Skip the Saturation Lookup Where No Liquid Vapor Equilibrium Exists======================
        if pressure >= self.get_output(Output::CriticalPressure)? {
            return Ok(Region::SinglePhase);
        }

        // Check the Liquid Side First to Save the Vapor Flash for Liquid States====================
        self.set_state(Pair::PressureQuality, pressure, 0.0)?;
        let h_liquid = self.get_output(Output::Enthalpy)?;
        if enthalpy < h_liquid {
            return Ok(Region::SinglePhase);
        }
        let rho_liquid = self.get_output(Output::Density)?;

        // Check the Vapor Side=====================================================================
        self.set_state(Pair::PressureQuality, pressure, 1.0)?;
        let h_vapor = self.get_output(Output::Enthalpy)?;
        if enthalpy > h_vapor {
            return Ok(Region::SinglePhase);
        }
        let rho_vapor = self.get_output(Output::Density)?;

        // Return the Saturated Properties Needed to Blend the Two Phases===========================
        Ok(Region::TwoPhase(Saturation {
            h_liquid,
            h_vapor,
            rho_liquid,
            rho_vapor,
        }))
    }

    //==============================================================================================
    // Function Name: get_glide
    //   Description: Test once whether the bubble and dew temperatures of the fluid differ
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_glide(&mut self) -> Result<bool, String> {
        // Return the Stored Result of an Earlier Test==============================================
        if let Some(glide) = self.glide {
            return Ok(glide);
        }

        // Compare the Saturated Liquid and Vapor Temperatures at One Pressure======================
        let pressure = GLIDE_PRESSURE_RATIO * self.get_output(Output::CriticalPressure)?;
        self.set_state(Pair::PressureQuality, pressure, 0.0)?;
        let t_liquid = self.get_output(Output::Temperature)?;
        self.set_state(Pair::PressureQuality, pressure, 1.0)?;
        let t_vapor = self.get_output(Output::Temperature)?;
        let glide = (t_vapor - t_liquid).abs() > GLIDE_TOLERANCE * t_liquid;

        self.glide = Some(glide);
        Ok(glide)
    }

    //==============================================================================================
    // Function Name: get_two_phase_slope_of
    //   Description: Select the density slope of a two phase state for the fluid
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_two_phase_slope_of(
        &mut self,
        pressure: f64,
        quality: f64,
        saturation: &Saturation,
    ) -> Result<f64, String> {
        if self.get_glide()? {
            // Differentiate the Saturated Properties Where the Library Formula Does Not Hold=======
            return self.get_saturation_slope(pressure, quality, saturation);
        }

        // Read the Two Phase Derivative of the Library=============================================
        self.set_state(Pair::PressureQuality, pressure, quality)?;
        self.get_two_phase_slope()
    }

    //==============================================================================================
    // Function Name: get_saturation_slope
    //   Description: Calculate the density slope at constant enthalpy from saturated properties
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_saturation_slope(
        &mut self,
        pressure: f64,
        quality: f64,
        saturation: &Saturation,
    ) -> Result<f64, String> {
        // Step Both Ways around the Pressure and Stay Below the Critical Pressure==================
        let critical = self.get_output(Output::CriticalPressure)?;
        let step = (SLOPE_STEP_RATIO * pressure).min(0.5 * (critical - pressure));

        // Read the Saturated Enthalpies and Specific Volumes on Both Sides=========================
        let (h_liquid_up, v_liquid_up) = self.get_saturated(pressure + step, 0.0)?;
        let (h_liquid_down, v_liquid_down) = self.get_saturated(pressure - step, 0.0)?;
        let (h_vapor_up, v_vapor_up) = self.get_saturated(pressure + step, 1.0)?;
        let (h_vapor_down, v_vapor_down) = self.get_saturated(pressure - step, 1.0)?;

        // Calculate the Slopes along the Saturation Lines by Central Differences===================
        let span = 2.0 * step;
        let d_h_liquid = (h_liquid_up - h_liquid_down) / span;
        let d_h_vapor = (h_vapor_up - h_vapor_down) / span;
        let d_v_liquid = (v_liquid_up - v_liquid_down) / span;
        let d_v_vapor = (v_vapor_up - v_vapor_down) / span;

        // Differentiate the Mixture Specific Volume at Constant Enthalpy===========================
        let v_liquid = 1.0 / saturation.rho_liquid;
        let v_vapor = 1.0 / saturation.rho_vapor;
        let d_quality = -((1.0 - quality) * d_h_liquid + quality * d_h_vapor)
            / (saturation.h_vapor - saturation.h_liquid);
        let d_volume =
            d_v_liquid + quality * (d_v_vapor - d_v_liquid) + (v_vapor - v_liquid) * d_quality;

        // Convert the Slope of the Specific Volume into That of the Density========================
        let density = get_mixture_density(quality, saturation.rho_liquid, saturation.rho_vapor);
        Ok(-density * density * d_volume)
    }

    //==============================================================================================
    // Function Name: get_saturated
    //   Description: Read the enthalpy and the specific volume of a saturated state
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_saturated(&mut self, pressure: f64, quality: f64) -> Result<(f64, f64), String> {
        self.set_state(Pair::PressureQuality, pressure, quality)?;
        let enthalpy = self.get_output(Output::Enthalpy)?;
        let density = self.get_output(Output::Density)?;
        Ok((enthalpy, 1.0 / density))
    }
}

//==================================================================================================
// Tests
//==================================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    //==============================================================================================
    // Function Name: glide_belongs_to_the_pseudo_pure_fluid_only
    //   Description: Check the glide flag of pure fluids and of air
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    #[test]
    fn glide_belongs_to_the_pseudo_pure_fluid_only() {
        for (fluid, expected) in [
            ("CO2", false),
            ("Nitrogen", false),
            ("Methane", false),
            ("Hydrogen", false),
            ("Helium", false),
            ("Water", false),
            ("R134a", false),
            ("Air", true),
        ] {
            let mut state = AbstractState::new(fluid).unwrap();
            assert_eq!(state.get_glide().unwrap(), expected, "{fluid}");
            assert_eq!(state.clone().glide, Some(expected), "{fluid}");
        }
    }

    //==============================================================================================
    // Function Name: viscosity_branches_meet_at_the_switch
    //   Description: Check that both branches of the mixture viscosity coincide at the switch
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    #[test]
    fn viscosity_branches_meet_at_the_switch() {
        let (rho_liquid, rho_vapor, mu_liquid, mu_vapor) = (800.0, 20.0, 1.0e-4, 1.5e-5);
        let quality = |void: f64| void / (void + (1.0 - void) * rho_liquid / rho_vapor);
        let below = VOID_FRACTION_SWITCH * (1.0 - 1.0e-12);
        let above = VOID_FRACTION_SWITCH * (1.0 + 1.0e-12);
        let low = get_mixture_viscosity(
            quality(below),
            below,
            rho_liquid,
            rho_vapor,
            mu_liquid,
            mu_vapor,
        );
        let high = get_mixture_viscosity(
            quality(above),
            above,
            rho_liquid,
            rho_vapor,
            mu_liquid,
            mu_vapor,
        );
        assert!((high / low - 1.0).abs() < 1e-9, "{low} {high}");
    }
}
