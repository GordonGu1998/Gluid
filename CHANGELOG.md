# Changelog

## 0.3.1 (not released)

- `get_flow_state` flashes the enthalpy and the pressure first and skips the saturation lookup when
  the temperature lies above the critical temperature (1.1 times that for a fluid with a glide).
  The returned bits do not change; air at 300 K between 1 and 20 bar takes 20 instead of 32
  microseconds per call. A state below that temperature takes the unchanged path.

## 0.3.0

- Breaking: removed `AbstractState::get_two_phase_density_derivative` (0.2.1).
- Breaking: `set_state` replaces the state after a failed update with a new one (no last update),
  because CoolProp refuses every later update of a state whose flash failed.
- Added `get_flow_state(pressure, enthalpy, with_slope)` and `get_thermal_state(pressure,
  enthalpy)` with `FlowState` and `ThermalState`: density, viscosity, density slope at constant
  enthalpy, conductivity, specific heat, temperature and its slope in the single-phase and the
  homogeneous two-phase region (mixture rules of Arauz and San Andres, 1998). The saturation lines
  belong to the two-phase region. An enthalpy that the equation of state refuses at the edge of
  its range is moved just inside it and `limited` is set.
- The two-phase density slope is CoolProp's two-phase derivative (Thorade and Saadat, 2013, pure
  fluids) only for a fluid without a temperature glide and at or above the lowest temperature of
  the equation of state. Otherwise (air; carbon dioxide and helium below the triple point, where
  the derivative drifts by up to 30 per cent) it follows from central differences of the saturated
  properties. Across the dome of air the temperature runs linearly in the quality from the bubble
  to the dew point.
- Added `get_phase_data`, `PhaseData`, `MeltingLine`, `MeltingLineKind`, `SublimationLine`,
  `get_phase_boundary`, `PhaseBoundary`, `Phase` and `is_on_saturation_line`. For air the phase
  tables avoid the critical region between the maxcondentherm and the maxcondenbar (Lemmon et al.,
  2000), where the flash fails.
- New dependency `serde_json` (fluid data).

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
