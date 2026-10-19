# Changelog

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
