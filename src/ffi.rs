//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::ffi::{c_char, c_int, c_long};

//==================================================================================================
// Foreign Functions
//==================================================================================================
// Declare the C Interface of the Static CoolProp Library===========================================
unsafe extern "C" {
    pub fn AbstractState_factory(
        backend: *const c_char,
        fluids: *const c_char,
        errcode: *mut c_long,
        message: *mut c_char,
        length: c_long,
    ) -> c_long;

    pub fn AbstractState_free(
        handle: c_long,
        errcode: *mut c_long,
        message: *mut c_char,
        length: c_long,
    );

    pub fn AbstractState_update(
        handle: c_long,
        pair: c_long,
        value1: f64,
        value2: f64,
        errcode: *mut c_long,
        message: *mut c_char,
        length: c_long,
    );

    pub fn AbstractState_keyed_output(
        handle: c_long,
        param: c_long,
        errcode: *mut c_long,
        message: *mut c_char,
        length: c_long,
    ) -> f64;

    pub fn get_param_index(param: *const c_char) -> c_long;

    pub fn get_input_pair_index(pair: *const c_char) -> c_long;

    pub fn get_fluid_param_string_len(fluid: *const c_char, param: *const c_char) -> c_long;

    pub fn get_fluid_param_string(
        fluid: *const c_char,
        param: *const c_char,
        output: *mut c_char,
        length: c_int,
    ) -> c_long;
}
