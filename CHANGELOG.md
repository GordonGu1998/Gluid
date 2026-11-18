# Changelog

## 0.3.0

- Breaking: removed `AbstractState::get_two_phase_density_derivative` (0.2.1).
- Added `AbstractState::get_flow_state(pressure, enthalpy, with_slope)` and
  `AbstractState::get_thermal_state(pressure, enthalpy)` with the records `FlowState` and
  `ThermalState`: density, viscosity, density slope at constant enthalpy, conductivity, specific
  heat, temperature and temperature slope in the single-phase and the two-phase region. The
  two-phase state is the homogeneous mixture of the saturated liquid and vapor (mixture rules of
  Arauz and San Andres, 1998).
- The two-phase density slope is CoolProp's two-phase derivative. For a fluid with a temperature
  glide (air) CoolProp's derivative does not match its own saturated properties (0.4 to 30 per
  cent), so the slope follows from central differences of the saturated properties instead. The
  fluid is tested once for a glide; no fluid name is stored.
- The saturated liquid and vapor lines belong to the two-phase region.

## 0.2.1

- Added `AbstractState::get_two_phase_density_derivative()` (removed in 0.3.0).

## 0.2.0

- Breaking: `Pair` and `Output` are `#[non_exhaustive]`; a `match` on them needs a wildcard arm.
- Added `AbstractState::get_derivative(of, wrt, constant)` and the enum `Variable` (density,
  enthalpy, pressure, temperature): first partial derivatives of a single-phase state. A two-phase
  state and a repeated variable are refused.
- Added `Output::Prandtl`, the Prandtl number.
- The example prints every feature; the README compares Gluid with rfluids; `examples` joins the
  packaged files.

## 0.1.0

First release. CoolProp 8.0.0 (commit `ae81610e7d23efc57f9d051c8e70a4d66e87537f`) compiled from
vendored source by `build.rs` and linked statically. `AbstractState`, `Pair`, `Output`,
`get_fluid_json` and `COOLPROP_VERSION`.
