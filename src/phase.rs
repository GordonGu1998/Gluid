//==================================================================================================
// Import
//==================================================================================================
// Import the State Its Enumerations and the Fluid Data from the Crate Root=========================
use crate::{AbstractState, Output, Pair, get_fluid_json};

//==================================================================================================
// Constants
//==================================================================================================
// Define the Relative Pressure Tolerance to Recognize a State on the Saturation Line===============
const SATURATION_TOLERANCE: f64 = 1.0e-5;

// Define the Relative Mismatch Allowed between the Melting Line and the Triple Point Pressure======
const MELTING_LINE_TOLERANCE: f64 = 1.0e-2;

// Define the Sublimation Coefficients of Carbon Dioxide by Span and Wagner Equation 3.12===========
const SUBLIMATION_COEFFICIENT: [f64; 3] = [-14.740_846, 2.432_701_5, -5.306_177_8];

// Define the Sublimation Exponents of Carbon Dioxide by Span and Wagner Equation 3.12==============
const SUBLIMATION_EXPONENT: [f64; 3] = [1.0, 1.9, 2.9];

// Define the Fluid with a Sublimation Equation=====================================================
const SUBLIMATION_FLUID: &str = "CarbonDioxide";

// Define the Fluid Whose Triple Point in the Library Is the Lambda Point and Not a Triple Point====
const LAMBDA_FLUID: &str = "Helium";

// Define the Number of Pressures of Each Table of the Phase Boundaries=============================
const PHASE_TABLE_SIZE: usize = 128;

// Define the Margin of the Pressure Range of the Tables============================================
const PHASE_RANGE_MARGIN: f64 = 0.01;

// Define the Relative Gap to the Critical Pressure=================================================
const CRITICAL_GAP: f64 = 1e-4;

// Define the Bisection Steps and the Doublings of the Bracket of the Critical Enthalpy=============
const CRITICAL_BISECTIONS: usize = 30;
const CRITICAL_DOUBLINGS: usize = 40;

//==================================================================================================
// Types
//==================================================================================================
// Declare the Equation Types of the Melting Line of CoolProp=======================================
/// Equation form of a melting line in the fluid data of CoolProp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeltingLineKind {
    // Define the Simon Melting Equation Form=======================================================
    Simon,

    // Define the Polynomial Melting Equation Form==================================================
    PolynomialInTr,

    // Define the Power Melting Equation Form=======================================================
    PolynomialInTheta,
}

// Declare the Melting Line Equation Read from the Fluid Data of CoolProp===========================
/// Melting line of a fluid, read from the fluid data of CoolProp, in kelvin and pascal.
#[derive(Clone, Debug, PartialEq)]
pub struct MeltingLine {
    // Define the Equation Type=====================================================================
    pub kind: MeltingLineKind,

    // Define the Reference Temperature T0 in Kelvin------------------------------------------------
    pub t_ref: f64,

    // Define the Reference Pressure p0 in Pascal---------------------------------------------------
    pub p_ref: f64,

    // Define the Coefficients ai with One Entry for Simon------------------------------------------
    pub coefficient: Vec<f64>,

    // Define the Exponents ti with One Entry for Simon as the Exponent c---------------------------
    pub exponent: Vec<f64>,

    // Define the Lowest Temperature of Validity in Kelvin------------------------------------------
    pub t_min: f64,

    // Define the Highest Temperature of Validity in Kelvin-----------------------------------------
    pub t_max: f64,
}

// Declare the Sublimation Line of Carbon Dioxide Anchored at the Triple Point======================
/// Sublimation line of carbon dioxide (Span and Wagner, 1996, Eq. 3.12), in kelvin and pascal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SublimationLine {
    // Define the Triple Point Temperature in Kelvin================================================
    pub t_triple: f64,

    // Define the Triple Point Pressure in Pascal===================================================
    pub p_triple: f64,
}

// Declare the Data of the Phase Diagram of a Fluid in Kelvin and Pascal============================
/// Critical point, lowest temperature, triple point, melting and sublimation lines of a fluid.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseData {
    // Define the Critical Temperature in Kelvin====================================================
    pub critical_temperature: f64,

    // Define the Critical Pressure in Pascal=======================================================
    pub critical_pressure: f64,

    // Define the Lowest Temperature of the Equation of State in Kelvin=============================
    pub min_temperature: f64,

    // Define the Triple Point in Kelvin and Pascal None for a Fluid without a Physical One=========
    pub triple: Option<(f64, f64)>,

    // Define the Melting Line None for a Fluid without a Usable One================================
    pub melting_line: Option<MeltingLine>,

    // Define the Sublimation Line None for a Fluid without an Equation=============================
    pub sublimation_line: Option<SublimationLine>,
}

// Declare the Phase of a State in the Pressure Enthalpy Plane======================================
/// Phase of a state, classified from a [`PhaseBoundary`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Supercritical,
    Gas,
    Liquid,
    TwoPhase,
}

// Declare the Tables of the Phase Boundaries over a Pressure Range=================================
/// Saturated enthalpies below and the enthalpy at the critical temperature above the critical
/// pressure, tabulated over a pressure range, in pascal and joule per kilogram.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseBoundary {
    // Define the Critical Pressure of the Fluid in Pascal==========================================
    pub p_critical: f64,

    // Define the Pressures below the Critical One and the Saturated Liquid and Vapor Enthalpies====
    pub p_sub: Vec<f64>,
    pub h_liquid: Vec<f64>,
    pub h_vapor: Vec<f64>,

    // Define the Pressures above the Critical One and the Enthalpy at the Critical Temperature=====
    pub p_super: Vec<f64>,
    pub h_critical: Vec<f64>,
}

// Declare the Entries of the Fluid Data of CoolProp Used Here======================================
struct FluidInfo {
    name: String,
    pseudo_pure: bool,
    melting_line: Option<MeltingLine>,
    top_saturation: Option<f64>,
    max_saturation: Option<f64>,
}

//==================================================================================================
// Lines
//==================================================================================================
impl MeltingLine {
    //==============================================================================================
    // Function Name: get_pressure
    //   Description: Evaluate the melting pressure of the line at a temperature
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Melting pressure in pascal at a temperature in kelvin.
    pub fn get_pressure(&self, temperature: f64) -> f64 {
        // Evaluate the Equation of the Line Type===================================================
        let ratio = temperature / self.t_ref;

        match self.kind {
            MeltingLineKind::Simon => {
                self.p_ref + self.coefficient[0] * (ratio.powf(self.exponent[0]) - 1.0)
            }
            MeltingLineKind::PolynomialInTr => {
                let sum: f64 = self
                    .coefficient
                    .iter()
                    .zip(&self.exponent)
                    .map(|(a, exponent)| a * (ratio.powf(*exponent) - 1.0))
                    .sum();

                self.p_ref * (1.0 + sum)
            }
            MeltingLineKind::PolynomialInTheta => {
                let theta = (ratio - 1.0).max(0.0);
                let sum: f64 = self
                    .coefficient
                    .iter()
                    .zip(&self.exponent)
                    .map(|(a, exponent)| a * theta.powf(*exponent))
                    .sum();

                self.p_ref * (1.0 + sum)
            }
        }
    }
}

impl SublimationLine {
    //==============================================================================================
    // Function Name: get_pressure
    //   Description: Evaluate the sublimation pressure of carbon dioxide at a temperature
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Sublimation pressure in pascal at a temperature in kelvin, or `None` above the triple
    /// point.
    pub fn get_pressure(&self, temperature: f64) -> Option<f64> {
        if temperature <= 0.0 || temperature > self.t_triple {
            return None;
        }

        let theta = 1.0 - temperature / self.t_triple;
        let sum: f64 = SUBLIMATION_COEFFICIENT
            .iter()
            .zip(SUBLIMATION_EXPONENT)
            .map(|(a, exponent)| a * theta.powf(exponent))
            .sum();

        Some(self.p_triple * (self.t_triple / temperature * sum).exp())
    }
}

impl PhaseBoundary {
    //==============================================================================================
    // Function Name: get_phase
    //   Description: Classify the phase at a pressure and an enthalpy from the tables
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Phase at a pressure and an enthalpy inside the tabulated pressure range.
    pub fn get_phase(&self, pressure: f64, enthalpy: f64) -> Phase {
        // Split the Pressures above the Critical One at the Critical Temperature===================
        if pressure >= self.p_critical {
            let h_critical = get_table_value(&self.p_super, &self.h_critical, pressure);
            return if enthalpy >= h_critical {
                Phase::Supercritical
            } else {
                Phase::Liquid
            };
        }

        // Compare the Enthalpy with the Saturated Liquid and the Saturated Vapor===================
        if enthalpy <= get_table_value(&self.p_sub, &self.h_liquid, pressure) {
            Phase::Liquid
        } else if enthalpy >= get_table_value(&self.p_sub, &self.h_vapor, pressure) {
            Phase::Gas
        } else {
            Phase::TwoPhase
        }
    }
}

//==================================================================================================
// Function Name: get_table_value
//   Description: Interpolate a table linearly in the pressure and hold its ends outside the range
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_table_value(p_table: &[f64], value: &[f64], p: f64) -> f64 {
    let upper = p_table.partition_point(|&entry| entry < p);
    if upper == 0 {
        return value[0];
    }
    if upper == p_table.len() {
        return value[upper - 1];
    }
    let weight = (p - p_table[upper - 1]) / (p_table[upper] - p_table[upper - 1]);
    value[upper - 1] + weight * (value[upper] - value[upper - 1])
}

//==================================================================================================
// Function Name: get_melting_line
//   Description: Read the melting line equation from the fluid data of CoolProp
//     Edit Date: Nov 18th, 2026.
//==================================================================================================
fn get_melting_line(fluid: &serde_json::Value) -> Option<MeltingLine> {
    // Locate the Single Melting Line Part in the Ancillaries of the Fluid==========================
    let line = fluid.get("ANCILLARIES")?.get("melting_line")?;
    let parts = line.get("parts")?.as_array()?;
    if parts.len() != 1 {
        return None;
    }
    let part = &parts[0];

    // Read the Equation Type and Its Coefficients and Exponents====================================
    let kind = match line.get("type")?.as_str()? {
        "Simon" => MeltingLineKind::Simon,
        "polynomial_in_Tr" => MeltingLineKind::PolynomialInTr,
        "polynomial_in_Theta" => MeltingLineKind::PolynomialInTheta,
        _ => return None,
    };
    let numbers = |key: &str| -> Option<Vec<f64>> {
        match part.get(key)? {
            serde_json::Value::Array(items) => items.iter().map(|item| item.as_f64()).collect(),
            other => Some(vec![other.as_f64()?]),
        }
    };

    // Map the Simon Exponent c to the Shared Exponent List=========================================
    let exponent = match kind {
        MeltingLineKind::Simon => numbers("c")?,
        _ => numbers("t")?,
    };
    let coefficient = numbers("a")?;
    if coefficient.len() != exponent.len() {
        return None;
    }

    // Return the Melting Line======================================================================
    Some(MeltingLine {
        kind,
        t_ref: part.get("T_0")?.as_f64()?,
        p_ref: part.get("p_0")?.as_f64()?,
        coefficient,
        exponent,
        t_min: part.get("T_min")?.as_f64()?,
        t_max: part.get("T_max")?.as_f64()?,
    })
}

//==================================================================================================
// Phase Data
//==================================================================================================
impl AbstractState {
    //==============================================================================================
    // Function Name: get_phase_data
    //   Description: Read the critical point the triple point and the melting and sublimation lines
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Read the data of the phase diagram of the fluid.
    ///
    /// The triple point is `None` for a pseudo-pure fluid (air) and for helium, whose triple
    /// point in CoolProp is the lambda point. The melting line is read from the fluid data of
    /// CoolProp; it is `None` for a pseudo-pure fluid and when it misses the triple point
    /// pressure by more than one per cent (the high-pressure fit of hydrogen). Only carbon
    /// dioxide has a sublimation line (Span and Wagner, 1996, Eq. 3.12).
    pub fn get_phase_data(&mut self) -> Result<PhaseData, String> {
        // Read the Fluid Data and the Fixed Points=================================================
        let info = self.get_fluid_info()?;
        let critical_temperature = self.get_output(Output::CriticalTemperature)?;
        let critical_pressure = self.get_output(Output::CriticalPressure)?;
        let min_temperature = self.get_output(Output::MinTemperature)?;
        let triple = if info.pseudo_pure || info.name == LAMBDA_FLUID {
            None
        } else {
            Some((
                self.get_output(Output::TripleTemperature)?,
                self.get_output(Output::TriplePressure)?,
            ))
        };

        // Reject a Melting Line Missing the Triple Point as the High Pressure Fit of Hydrogen======
        let mut melting_line = if info.pseudo_pure {
            None
        } else {
            info.melting_line
        };
        if let (Some(line), Some((t_triple, p_triple))) = (&melting_line, triple)
            && (line.get_pressure(t_triple) - p_triple).abs() > MELTING_LINE_TOLERANCE * p_triple
        {
            melting_line = None;
        }

        // Anchor the Sublimation Line of Carbon Dioxide at the Triple Point========================
        let sublimation_line = match triple {
            Some((t_triple, p_triple)) if info.name == SUBLIMATION_FLUID => {
                Some(SublimationLine { t_triple, p_triple })
            }
            _ => None,
        };

        Ok(PhaseData {
            critical_temperature,
            critical_pressure,
            min_temperature,
            triple,
            melting_line,
            sublimation_line,
        })
    }

    //==============================================================================================
    // Function Name: is_on_saturation_line
    //   Description: Check whether a pressure and a temperature describe a saturated state
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Whether the pressure lies on the saturation line at the temperature, within a relative
    /// tolerance of 1e-5. The call changes the current state.
    pub fn is_on_saturation_line(&mut self, pressure: f64, temperature: f64) -> bool {
        self.set_state(Pair::QualityTemperature, 0.0, temperature)
            .and_then(|()| self.get_output(Output::Pressure))
            .is_ok_and(|p_saturation| {
                (p_saturation - pressure).abs() <= SATURATION_TOLERANCE * pressure
            })
    }

    //==============================================================================================
    // Function Name: get_phase_boundary
    //   Description: Tabulate the enthalpies of the phase boundaries over a pressure range
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    /// Tabulate the saturated enthalpies below and the enthalpy at the critical temperature
    /// above the critical pressure, over a pressure range widened by one per cent, for
    /// [`PhaseBoundary::get_phase`].
    ///
    /// For a pseudo-pure fluid (air) the saturated table ends at the maxcondentherm, above
    /// which the dew line of CoolProp is the bubble line (Lemmon et al., 2000), and the table
    /// above the critical pressure starts above the maxcondenbar, below which the flash fails.
    /// The call changes the current state.
    pub fn get_phase_boundary(&mut self, p_range: (f64, f64)) -> Result<PhaseBoundary, String> {
        let info = self.get_fluid_info()?;
        let p_critical = self.get_output(Output::CriticalPressure)?;
        let p_top = info
            .top_saturation
            .map_or(p_critical, |p| p.min(p_critical));
        let p_bottom = info
            .max_saturation
            .map_or(p_critical, |p| p.max(p_critical));
        let p_low = p_range.0 * (1.0 - PHASE_RANGE_MARGIN);
        let p_high = p_range.1 * (1.0 + PHASE_RANGE_MARGIN);
        let mut boundary = PhaseBoundary {
            p_critical,
            p_sub: Vec::new(),
            h_liquid: Vec::new(),
            h_vapor: Vec::new(),
            p_super: Vec::new(),
            h_critical: Vec::new(),
        };

        // Tabulate the Saturated Enthalpies Crowding the Pressures toward the Critical Point=======
        if p_low < p_critical {
            let num_point = PHASE_TABLE_SIZE - 1;
            for i in 0..num_point {
                let s = i as f64 / num_point as f64;
                let p = p_top - (p_top - p_low) * (1.0 - s) * (1.0 - s);
                self.set_state(Pair::PressureQuality, p, 0.0)?;
                let h_liquid = self.get_output(Output::Enthalpy)?;
                self.set_state(Pair::PressureQuality, p, 1.0)?;
                let h_vapor = self.get_output(Output::Enthalpy)?;
                boundary.p_sub.push(p);
                boundary.h_liquid.push(h_liquid);
                boundary.h_vapor.push(h_vapor);
            }
        }

        // Tabulate the Enthalpy at the Critical Temperature Starting from the Critical Point=======
        if p_high >= p_critical {
            let p_start = p_bottom * (1.0 + CRITICAL_GAP);
            self.set_state(Pair::PressureQuality, p_top * (1.0 - CRITICAL_GAP), 0.0)?;
            let h_liquid = self.get_output(Output::Enthalpy)?;
            self.set_state(Pair::PressureQuality, p_top * (1.0 - CRITICAL_GAP), 1.0)?;
            let h_vapor = self.get_output(Output::Enthalpy)?;
            let step = 0.1 * (h_vapor - h_liquid).abs().max(1.0e3);
            let mut last = 0.5 * (h_liquid + h_vapor);
            for i in 0..PHASE_TABLE_SIZE {
                let p = p_start
                    + (p_high - p_start).max(0.0) * i as f64 / (PHASE_TABLE_SIZE - 1) as f64;
                last = self.get_critical_enthalpy(p, (last, step))?;
                boundary.p_super.push(p);
                boundary.h_critical.push(last);
            }
        }

        Ok(boundary)
    }

    //==============================================================================================
    // Function Name: get_critical_enthalpy
    //   Description: Find the enthalpy at the critical temperature at a pressure above the critical
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_critical_enthalpy(&mut self, p: f64, seed: (f64, f64)) -> Result<f64, String> {
        let t_critical = self.get_output(Output::CriticalTemperature)?;
        let (h_seed, step) = seed;
        let is_below = |state: &mut Self, h: f64| -> Result<bool, String> {
            state.set_enthalpy_pressure(h, p)?;
            Ok(state.get_output(Output::Temperature)? < t_critical)
        };

        // Bracket the Critical Enthalpy by Doubling the Step from the Seed=========================
        let (mut low, mut high) = (h_seed, h_seed);
        let mut width = step;
        if is_below(self, h_seed)? {
            for _ in 0..CRITICAL_DOUBLINGS {
                high = low + width;
                if !is_below(self, high)? {
                    break;
                }
                low = high;
                width *= 2.0;
            }
        } else {
            for _ in 0..CRITICAL_DOUBLINGS {
                low = high - width;
                if is_below(self, low)? {
                    break;
                }
                high = low;
                width *= 2.0;
            }
        }

        // Bisect the Bracket Because the Temperature Rises with the Enthalpy at a Pressure=========
        for _ in 0..CRITICAL_BISECTIONS {
            let middle = 0.5 * (low + high);
            if is_below(self, middle)? {
                low = middle;
            } else {
                high = middle;
            }
        }

        Ok(0.5 * (low + high))
    }

    //==============================================================================================
    // Function Name: get_fluid_info
    //   Description: Read the name the pseudo pure flag the melting line and the saturation limits
    //     Edit Date: Nov 18th, 2026.
    //==============================================================================================
    fn get_fluid_info(&self) -> Result<FluidInfo, String> {
        // Parse the Fluid Data of the Library======================================================
        let name = self.fluid.to_string_lossy();
        let json = get_fluid_json(&name).ok_or(format!("CoolProp has no data for {name}"))?;
        let value: serde_json::Value =
            serde_json::from_str(&json).map_err(|error| error.to_string())?;
        let fluid = if value.is_array() { &value[0] } else { &value };
        let eos = &fluid["EOS"][0];
        let state_pressure = |key: &str| eos["STATES"][key]["p"].as_f64();

        // Read the Entries Used by the Phase Data and the Phase Boundaries=========================
        Ok(FluidInfo {
            name: fluid["INFO"]["NAME"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            pseudo_pure: eos["pseudo_pure"].as_bool().unwrap_or(false),
            melting_line: get_melting_line(fluid),
            top_saturation: state_pressure("temperature_max_sat"),
            max_saturation: state_pressure("pressure_max_sat"),
        })
    }
}
