#![doc = include_str!("../README.md")]

//==================================================================================================
// Modules
//==================================================================================================
// Declare the Raw Interface of the Static CoolProp Library=========================================
mod ffi;

//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::ffi::{CStr, CString, c_char, c_int, c_long};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Import the Raw Interface of the Static CoolProp Library==========================================
use crate::ffi::{
    AbstractState_factory, AbstractState_free, AbstractState_keyed_output, AbstractState_update,
    get_fluid_param_string, get_fluid_param_string_len, get_input_pair_index, get_param_index,
};

//==================================================================================================
// Constants
//==================================================================================================
// Declare the Version of the CoolProp Source That Is Linked========================================
pub const COOLPROP_VERSION: &str = env!("GLUID_COOLPROP_VERSION");

// Define the Capacity of the Buffer Holding an Error Message of the Library========================
const MESSAGE_CAPACITY: usize = 512;

// Define the Backend of the Library Used for Every Pure Fluid======================================
const BACKEND: &CStr = c"HEOS";

// Define the Names of the Input Pairs in the Order of the Pair Enumeration=========================
const PAIR_NAME: [&CStr; 4] = [c"PT_INPUTS", c"HmassP_INPUTS", c"PQ_INPUTS", c"QT_INPUTS"];

// Define the Names of the Outputs in the Order of the Output Enumeration===========================
const OUTPUT_NAME: [&CStr; 14] = [
    c"Dmass",
    c"Hmass",
    c"Cpmass",
    c"viscosity",
    c"conductivity",
    c"T",
    c"P",
    c"isothermal_compressibility",
    c"isobaric_expansion_coefficient",
    c"pcrit",
    c"Tcrit",
    c"Tmin",
    c"Ttriple",
    c"ptriple",
];

//==================================================================================================
// Synchronization
//==================================================================================================
// Declare the Lock around the Global State of the Library==========================================
static LIBRARY: RwLock<()> = RwLock::new(());

//==================================================================================================
// Function Name: get_shared
//   Description: Take the lock shared for the calls on one state of a pure fluid
//     Edit Date: Oct 15th, 2026.
//==================================================================================================
fn get_shared() -> RwLockReadGuard<'static, ()> {
    LIBRARY.read().unwrap_or_else(PoisonError::into_inner)
}

//==================================================================================================
// Function Name: get_exclusive
//   Description: Take the lock exclusive for the calls on the global state of the library
//     Edit Date: Oct 15th, 2026.
//==================================================================================================
fn get_exclusive() -> RwLockWriteGuard<'static, ()> {
    LIBRARY.write().unwrap_or_else(PoisonError::into_inner)
}

//==================================================================================================
// Error Buffer
//==================================================================================================
// Declare the Buffer Receiving the Error Code and the Message of a Call============================
struct ErrorBuffer {
    // Define the Error Code Zero for a Call That Succeeded=========================================
    code: c_long,

    // Define the Message of the Error with a Terminating Zero======================================
    message: [u8; MESSAGE_CAPACITY],
}

impl ErrorBuffer {
    //==============================================================================================
    // Function Name: new
    //   Description: Create an empty buffer for a call
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn new() -> Self {
        Self {
            code: 0,
            message: [0; MESSAGE_CAPACITY],
        }
    }

    //==============================================================================================
    // Function Name: get_parts
    //   Description: Get the pointers and the length handed to a call of the library
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn get_parts(&mut self) -> (*mut c_long, *mut c_char, c_long) {
        (
            &raw mut self.code,
            self.message.as_mut_ptr().cast(),
            MESSAGE_CAPACITY as c_long,
        )
    }

    //==============================================================================================
    // Function Name: get_result
    //   Description: Turn the buffer into an error carrying the message of the library
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn get_result(&self) -> Result<(), String> {
        if self.code == 0 {
            return Ok(());
        }
        let text = CStr::from_bytes_until_nul(&self.message)
            .map(|message| message.to_string_lossy().trim().to_string())
            .unwrap_or_default();
        if text.is_empty() {
            Err(format!("CoolProp failed with the error code {}", self.code))
        } else {
            Err(text)
        }
    }
}

//==================================================================================================
// Keys
//==================================================================================================
// Declare the Input Pairs Taken by an Update of a State============================================
/// Input pair of an update, with the two values given in the order of the name.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pair {
    // Define Pressure and Temperature in This Order================================================
    PressureTemperature,

    // Define Specific Enthalpy and Pressure in This Order==========================================
    EnthalpyPressure,

    // Define Pressure and Quality in This Order====================================================
    PressureQuality,

    // Define Quality and Temperature in This Order=================================================
    QualityTemperature,
}

// Declare the Outputs Read from a State============================================================
/// Output read from a state, in SI units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    // Define the Density in Kilogram per Cubic Meter===============================================
    Density,

    // Define the Specific Enthalpy in Joule per Kilogram===========================================
    Enthalpy,

    // Define the Specific Heat Capacity at Constant Pressure in Joule per Kilogram per Kelvin======
    SpecificHeat,

    // Define the Dynamic Viscosity in Pascal Second================================================
    Viscosity,

    // Define the Thermal Conductivity in Watt per Meter per Kelvin=================================
    Conductivity,

    // Define the Temperature in Kelvin=============================================================
    Temperature,

    // Define the Pressure in Pascal================================================================
    Pressure,

    // Define the Isothermal Compressibility in per Pascal==========================================
    IsothermalCompressibility,

    // Define the Isobaric Expansion Coefficient in per Kelvin======================================
    IsobaricExpansion,

    // Define the Critical Pressure in Pascal=======================================================
    CriticalPressure,

    // Define the Critical Temperature in Kelvin====================================================
    CriticalTemperature,

    // Define the Lowest Temperature of the Equation of State in Kelvin=============================
    MinTemperature,

    // Define the Triple Temperature in Kelvin======================================================
    TripleTemperature,

    // Define the Triple Pressure in Pascal=========================================================
    TriplePressure,
}

impl Output {
    //==============================================================================================
    // Function Name: is_positive
    //   Description: Check whether the output must be greater than zero
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn is_positive(self) -> bool {
        !matches!(
            self,
            Self::Enthalpy | Self::IsothermalCompressibility | Self::IsobaricExpansion
        )
    }
}

// Declare the Integer Keys of the Library for the Pairs and the Outputs============================
struct Keys {
    // Define the Keys of the Input Pairs===========================================================
    pair: [c_long; PAIR_NAME.len()],

    // Define the Keys of the Outputs===============================================================
    output: [c_long; OUTPUT_NAME.len()],
}

//==================================================================================================
// Function Name: get_keys
//   Description: Look up the integer keys of the library once under the exclusive lock
//     Edit Date: Oct 15th, 2026.
//==================================================================================================
fn get_keys() -> Result<&'static Keys, String> {
    // Look Up Every Name Once and Keep the Result==================================================
    static KEYS: OnceLock<Result<Keys, String>> = OnceLock::new();
    let keys = KEYS.get_or_init(|| {
        let _guard = get_exclusive();
        let mut keys = Keys {
            pair: [0; PAIR_NAME.len()],
            output: [0; OUTPUT_NAME.len()],
        };
        for (key, name) in keys.pair.iter_mut().zip(PAIR_NAME) {
            // Call the Library with a Name That Ends with a Zero===================================
            *key = unsafe { get_input_pair_index(name.as_ptr()) };
            if *key < 0 {
                return Err(format!(
                    "CoolProp has no input pair {}",
                    name.to_string_lossy()
                ));
            }
        }
        for (key, name) in keys.output.iter_mut().zip(OUTPUT_NAME) {
            // Call the Library with a Name That Ends with a Zero===================================
            *key = unsafe { get_param_index(name.as_ptr()) };
            if *key < 0 {
                return Err(format!("CoolProp has no output {}", name.to_string_lossy()));
            }
        }
        Ok(keys)
    });

    // Return the Keys or the Message of the Failed Look Up=========================================
    keys.as_ref().map_err(Clone::clone)
}

//==================================================================================================
// Function Name: get_fluid_json
//   Description: Read the JSON description of a fluid from the library
//     Edit Date: Oct 15th, 2026.
//==================================================================================================
/// Read the JSON description of a fluid, `None` for an unknown fluid.
pub fn get_fluid_json(fluid: &str) -> Option<String> {
    // Take the Global State of the Library Exclusively=============================================
    let fluid = CString::new(fluid).ok()?;
    let _guard = get_exclusive();

    // Read the Length and Then the Text============================================================
    // Call the Library with Names That End with a Zero=============================================
    let length = unsafe { get_fluid_param_string_len(fluid.as_ptr(), c"JSON".as_ptr()) };
    if length < 0 {
        return None;
    }
    let mut buffer = vec![0u8; length as usize + 1];
    // Call the Library with a Buffer of the Length Told to It======================================
    let status = unsafe {
        get_fluid_param_string(
            fluid.as_ptr(),
            c"JSON".as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len() as c_int,
        )
    };
    if status == 0 {
        return None;
    }
    CStr::from_bytes_until_nul(&buffer)
        .ok()
        .map(|text| text.to_string_lossy().into_owned())
}

//==================================================================================================
// State
//==================================================================================================
// Declare the Last Update of a State to Replay It in a Copy and to Return Its Inputs===============
#[derive(Clone, Copy)]
struct Update {
    // Define the Input Pair of the Update==========================================================
    pair: Pair,

    // Define the First Value in the Order of the Pair==============================================
    value1: f64,

    // Define the Second Value in the Order of the Pair=============================================
    value2: f64,
}

// Declare the State of One Pure Fluid with the Backend HEOS========================================
/// State of one pure fluid. Cloning replays the last update on a new state.
pub struct AbstractState {
    // Define the Handle of the State in the Library================================================
    handle: c_long,

    // Define the Name of the Fluid=================================================================
    fluid: CString,

    // Define the Last Update None before the First One=============================================
    update: Option<Update>,
}

//==================================================================================================
// Implementation
//==================================================================================================
impl AbstractState {
    //==============================================================================================
    // Default Structure
    //==============================================================================================
    /// Create the state of a pure fluid by the name CoolProp knows it by.
    pub fn new(fluid: &str) -> Result<Self, String> {
        // Create the State of the Fluid under the Shared Lock======================================
        let fluid = CString::new(fluid).map_err(|error| error.to_string())?;
        get_keys()?;
        let mut buffer = ErrorBuffer::new();
        let (code, message, length) = buffer.get_parts();
        let handle = {
            let _guard = get_shared();
            let (backend, name) = (BACKEND.as_ptr(), fluid.as_ptr());
            // Call the Library with Names That End with a Zero and with Pointers into the Buffer===
            unsafe { AbstractState_factory(backend, name, code, message, length) }
        };
        buffer.get_result()?;

        // Return the State=========================================================================
        Ok(Self {
            handle,
            fluid,
            update: None,
        })
    }

    //==============================================================================================
    // Function Name: set_state
    //   Description: Update the state from two inputs in the order of the pair
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    /// Update the state from two inputs in the order of the pair.
    pub fn set_state(&mut self, pair: Pair, value1: f64, value2: f64) -> Result<(), String> {
        // Update the State under the Shared Lock===================================================
        let key = get_keys()?.pair[pair as usize];
        let mut buffer = ErrorBuffer::new();
        let (code, message, length) = buffer.get_parts();
        {
            let _guard = get_shared();
            let handle = self.handle;
            // Call the Library with a Live Handle and with Pointers into the Buffer================
            unsafe { AbstractState_update(handle, key, value1, value2, code, message, length) };
        }
        buffer.get_result()?;

        // Keep the Update for the Inputs and the Copies============================================
        self.update = Some(Update {
            pair,
            value1,
            value2,
        });
        Ok(())
    }

    //==============================================================================================
    // Function Name: get_output
    //   Description: Read an output of the current state and return an input as given
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    /// Read an output, an error for a failed call or a value that is not valid.
    pub fn get_output(&mut self, output: Output) -> Result<f64, String> {
        // Take an Input Exactly as Given to the Update or Read the Output from the Library=========
        let value = match self.get_input(output) {
            Some(value) => value,
            None => self.get_computed(output)?,
        };

        // Return the Value Only If It Is a Number and Positive Where Physics Needs It==============
        if value.is_finite() && (value > 0.0 || !output.is_positive()) {
            Ok(value)
        } else {
            Err(format!(
                "CoolProp returned no valid value {value} for {output:?}"
            ))
        }
    }

    //==============================================================================================
    // Function Name: get_computed
    //   Description: Read an output of the current state from the library under the shared lock
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn get_computed(&mut self, output: Output) -> Result<f64, String> {
        let key = get_keys()?.output[output as usize];
        let mut buffer = ErrorBuffer::new();
        let (code, message, length) = buffer.get_parts();
        let value = {
            let _guard = get_shared();
            // Call the Library with a Live Handle and with Pointers into the Buffer================
            unsafe { AbstractState_keyed_output(self.handle, key, code, message, length) }
        };
        buffer.get_result()?;
        Ok(value)
    }

    //==============================================================================================
    // Function Name: get_input
    //   Description: Return the output that was an input of the last update
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn get_input(&self, output: Output) -> Option<f64> {
        let update = self.update?;
        match (update.pair, output) {
            (Pair::PressureTemperature, Output::Pressure) => Some(update.value1),
            (Pair::PressureTemperature, Output::Temperature) => Some(update.value2),
            (Pair::EnthalpyPressure, Output::Enthalpy) => Some(update.value1),
            (Pair::EnthalpyPressure, Output::Pressure) => Some(update.value2),
            (Pair::PressureQuality, Output::Pressure) => Some(update.value1),
            (Pair::QualityTemperature, Output::Temperature) => Some(update.value2),
            _ => None,
        }
    }
}

//==================================================================================================
// Copy
//==================================================================================================
impl Clone for AbstractState {
    //==============================================================================================
    // Function Name: clone
    //   Description: Create a new state of the fluid in the same state as the original
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn clone(&self) -> Self {
        let name = self.fluid.to_string_lossy();
        let mut copy = Self::new(&name).expect("the fluid of a valid state exists");
        if let Some(update) = self.update {
            copy.set_state(update.pair, update.value1, update.value2)
                .expect("the last update of a valid state is valid");
        }
        copy
    }
}

//==================================================================================================
// Release
//==================================================================================================
impl Drop for AbstractState {
    //==============================================================================================
    // Function Name: drop
    //   Description: Free the state in the library under the shared lock
    //     Edit Date: Oct 15th, 2026.
    //==============================================================================================
    fn drop(&mut self) {
        let mut buffer = ErrorBuffer::new();
        let (code, message, length) = buffer.get_parts();
        let _guard = get_shared();
        // Call the Library with a Live Handle and with Pointers into the Buffer====================
        unsafe { AbstractState_free(self.handle, code, message, length) };
    }
}
