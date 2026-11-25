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

// Define the Relative Distance inside the Range at Which a Limited Enthalpy Is Placed==============
const EDGE_MARGIN: f64 = 1.0e-4;

// Define the Most Times the Edge of the Range Is Read from the Message and Tried===================
const LIMIT_ATTEMPTS: usize = 4;

// Define the Temperature Ratio to the Critical Point above Which a Fluid with a Glide Is a Gas=====
const GLIDE_HOT_RATIO: f64 = 1.1;

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

    // Define Whether the Enthalpy Was Limited to the Range of the Equation of State================
    pub limited: bool,
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

    // Define Whether the Enthalpy Was Limited to the Range of the Equation of State================
    pub limited: bool,
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
// Function Name: get_limited_enthalpy
//   Description: Read the edge of the enthalpy range from a flash error and limit the enthalpy
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_limited_enthalpy(message: &str, enthalpy: f64) -> Option<f64> {
    // Read the Molar Enthalpy That Was Refused and the Edge of the Range from the Message==========
    let number_after = |key: &str| -> Option<f64> {
        let tail = &message[message.find(key)? + key.len()..];
        let end = tail
            .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
            .unwrap_or(tail.len());
        tail[..end].parse().ok()
    };
    let refused = number_after("Hmolar [")?;
    let (edge, inside) = if message.contains("is below the minimum value of") {
        (number_after("minimum value of ")?, 1.0)
    } else if message.contains("is above the maximum value of") {
        (number_after("maximum value of ")?, -1.0)
    } else {
        return None;
    };

    // Convert the Edge to Joules per Kilogram with the Ratio of the Two Enthalpies=================
    if refused == 0.0 || !(enthalpy / refused).is_finite() {
        return None;
    }
    let limit = edge * (enthalpy / refused);
    Some(limit + inside * EDGE_MARGIN * limit.abs().max(1.0))
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
    ///
    /// The slope of a single-phase state is CoolProp's derivative at constant enthalpy. In the
    /// two-phase region it is CoolProp's two-phase derivative (Thorade and Saadat, 2013), which
    /// assumes a pure fluid with one saturation temperature. It is used only for a fluid without a
    /// temperature glide and a saturation temperature at or above the lowest temperature of the
    /// equation of state. Otherwise (air, or a state below the triple point) the slope follows
    /// from central differences of the saturated properties. The saturation lines belong to the
    /// two-phase region (quality 0 and 1), and the slope jumps across them.
    ///
    /// An enthalpy that CoolProp refuses at the edge of the range of the equation of state is
    /// moved just inside that edge, and `limited` is set. The call changes the current state.
    pub fn get_flow_state(
        &mut self,
        pressure: f64,
        enthalpy: f64,
        with_slope: bool,
    ) -> Result<FlowState, String> {
        // Skip the Saturation Lookup of a State Hotter Than the Dome of the Fluid==================
        if let Some(state) = self.get_hot_flow_state(pressure, enthalpy, with_slope) {
            return Ok(state);
        }
        self.get_flow_state_by_region(pressure, enthalpy, with_slope)
    }

    //==============================================================================================
    // Function Name: get_flow_state_by_region
    //   Description: Calculate the flow state after the saturation lookup of the region
    //     Edit Date: Nov 24th, 2026.
    //==============================================================================================
    fn get_flow_state_by_region(
        &mut self,
        pressure: f64,
        enthalpy: f64,
        with_slope: bool,
    ) -> Result<FlowState, String> {
        match self.get_region(pressure, enthalpy)? {
            Region::SinglePhase => {
                // Flash the State Once and Read Only the Requested Properties======================
                let limited = self.set_enthalpy_pressure(enthalpy, pressure)?;
                self.get_single_phase_flow(limited, with_slope)
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
                    limited: false,
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
    /// weighted mean without latent heat. For a pure fluid the temperature is the saturation
    /// temperature and the slope is zero. For a fluid with a temperature glide (air) the
    /// temperature runs linearly in the quality from the bubble to the dew point, with the slope
    /// of that line against the enthalpy. A limited enthalpy sets `limited` as in
    /// [`AbstractState::get_flow_state`]. The call changes the current state.
    pub fn get_thermal_state(
        &mut self,
        pressure: f64,
        enthalpy: f64,
    ) -> Result<ThermalState, String> {
        match self.get_region(pressure, enthalpy)? {
            Region::SinglePhase => {
                // Flash the State Once and Read the Three Properties===============================
                let limited = self.set_enthalpy_pressure(enthalpy, pressure)?;

                let specific_heat = self.get_output(Output::SpecificHeat)?;

                Ok(ThermalState {
                    conductivity: self.get_output(Output::Conductivity)?,
                    specific_heat,
                    temperature: self.get_output(Output::Temperature)?,
                    temperature_slope: 1.0 / specific_heat,
                    limited,
                })
            }

            Region::TwoPhase(saturation) => {
                let quality = get_quality(enthalpy, saturation.h_liquid, saturation.h_vapor);

                // Read the Saturated Vapor Which Also Gives the Saturation Temperature=============
                self.set_state(Pair::PressureQuality, pressure, 1.0)?;
                let k_vapor = self.get_output(Output::Conductivity)?;
                let c_vapor = self.get_output(Output::SpecificHeat)?;
                let t_vapor = self.get_output(Output::Temperature)?;

                // Read the Saturated Liquid========================================================
                self.set_state(Pair::PressureQuality, pressure, 0.0)?;
                let k_liquid = self.get_output(Output::Conductivity)?;
                let c_liquid = self.get_output(Output::SpecificHeat)?;
                let t_liquid = self.get_output(Output::Temperature)?;

                // Interpolate the Temperature in the Quality across the Glide of the Dome==========
                let (temperature, temperature_slope) = if self.get_glide()? {
                    let gap = t_vapor - t_liquid;
                    (
                        t_liquid + quality * gap,
                        gap / (saturation.h_vapor - saturation.h_liquid),
                    )
                } else {
                    (t_vapor, 0.0)
                };

                Ok(ThermalState {
                    conductivity: get_mixture_conductivity(quality, k_liquid, k_vapor),
                    specific_heat: get_mixture_specific_heat(quality, c_liquid, c_vapor),
                    temperature,
                    temperature_slope,
                    limited: false,
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
    // Function Name: get_single_phase_flow
    //   Description: Read the flow properties of the flashed single phase state
    //     Edit Date: Nov 24th, 2026.
    //==============================================================================================
    fn get_single_phase_flow(
        &mut self,
        limited: bool,
        with_slope: bool,
    ) -> Result<FlowState, String> {
        let density = self.get_output(Output::Density)?;
        let viscosity = self.get_output(Output::Viscosity)?;
        let density_slope = if with_slope {
            Some(self.get_derivative(Variable::Density, Variable::Pressure, Variable::Enthalpy)?)
        } else {
            None
        };

        Ok(FlowState {
            density,
            viscosity,
            density_slope,
            limited,
        })
    }

    //==============================================================================================
    // Function Name: get_hot_flow_state
    //   Description: Flash a state and return its flow state if it lies above the dome
    //     Edit Date: Nov 24th, 2026.
    //==============================================================================================
    fn get_hot_flow_state(
        &mut self,
        pressure: f64,
        enthalpy: f64,
        with_slope: bool,
    ) -> Option<FlowState> {
        // Read the Margin of a Glide before the Flash because the Test Changes the State===========
        let ratio = if self.get_glide().ok()? {
            GLIDE_HOT_RATIO
        } else {
            1.0
        };

        // Flash the State and Leave Every Refusal to the Lookup of the Region======================
        let limited = self.set_enthalpy_pressure(enthalpy, pressure).ok()?;

        // Accept Only a Temperature above the Critical Point Plus the Margin=======================
        let t_critical = self.get_output(Output::CriticalTemperature).ok()?;
        if self.get_output(Output::Temperature).ok()? <= ratio * t_critical {
            return None;
        }
        self.get_single_phase_flow(limited, with_slope).ok()
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
        // Differentiate the Saturated Properties Where the Library Formula Does Not Hold===========
        if self.get_glide()? {
            return self.get_saturation_slope(pressure, quality, saturation);
        }
        self.set_state(Pair::PressureQuality, pressure, quality)?;
        let t_min = self.get_output(Output::MinTemperature)?;
        if self.get_output(Output::Temperature)? < t_min {
            return self.get_saturation_slope(pressure, quality, saturation);
        }

        // Read the Two Phase Derivative of the Library=============================================
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
    // Function Name: set_enthalpy_pressure
    //   Description: Flash the enthalpy and pressure and limit a refused enthalpy to the range
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    pub(crate) fn set_enthalpy_pressure(
        &mut self,
        enthalpy: f64,
        pressure: f64,
    ) -> Result<bool, String> {
        // Move a Refused Enthalpy inside the Edge the Flash Names and Try Again====================
        let mut value = enthalpy;
        let mut limited = false;
        for attempt in 0..LIMIT_ATTEMPTS {
            match self.set_state(Pair::EnthalpyPressure, value, pressure) {
                Ok(()) => return Ok(limited),
                Err(message) => match get_limited_enthalpy(&message, value) {
                    Some(edge) if attempt + 1 < LIMIT_ATTEMPTS => {
                        limited = true;
                        value = edge;
                    }
                    _ => return Err(message),
                },
            }
        }
        unreachable!("The loop returns at its last attempt")
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

    //==============================================================================================
    // Function Name: hot_shortcut_equals_the_lookup_of_the_region
    //   Description: Check that the shortcut above the dome returns the bits of the lookup
    //     Edit Date: Nov 24th, 2026.
    //==============================================================================================
    #[test]
    fn hot_shortcut_equals_the_lookup_of_the_region() {
        for (fluid, p_low, p_high, t_low, t_high) in [
            ("Air", 1.0e5, 6.0e6, 250.0, 450.0),
            ("Nitrogen", 1.0e5, 6.0e6, 100.0, 450.0),
            ("CO2", 1.0e5, 2.0e7, 220.0, 450.0),
            ("Methane", 1.0e5, 1.0e7, 100.0, 450.0),
            ("Hydrogen", 1.0e5, 1.0e7, 30.0, 450.0),
            ("Helium", 1.0e5, 1.0e6, 4.0, 450.0),
        ] {
            let mut state = AbstractState::new(fluid).unwrap();
            let mut hot = 0;
            for i in 0..=24 {
                for j in 0..=24 {
                    let pressure = p_low + (p_high - p_low) * (i as f64) / 24.0;
                    let t = t_low + (t_high - t_low) * (j as f64) / 24.0;
                    if state
                        .set_state(Pair::PressureTemperature, pressure, t)
                        .is_err()
                    {
                        continue;
                    }
                    let enthalpy = state.get_output(Output::Enthalpy).unwrap();
                    for with_slope in [false, true] {
                        let by_region =
                            state.get_flow_state_by_region(pressure, enthalpy, with_slope);
                        let shortcut = state.get_flow_state(pressure, enthalpy, with_slope);
                        assert_eq!(by_region, shortcut, "{fluid} {pressure} {t}");
                    }
                    if state
                        .get_hot_flow_state(pressure, enthalpy, false)
                        .is_some()
                    {
                        hot += 1;
                    }
                }
            }
            assert!(hot > 0, "{fluid} never took the shortcut");
        }
    }
}
