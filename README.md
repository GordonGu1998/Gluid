# Gluid

[![CI](https://github.com/GordonGu1998/Gluid/actions/workflows/ci.yml/badge.svg)](https://github.com/GordonGu1998/Gluid/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Fluid properties for Rust, built by `cargo build` alone.**

Gluid gives Rust programs the thermophysical properties of more than 130 pure and pseudo-pure
fluids (density, viscosity, conductivity, heat capacity, saturation, critical and triple points)
from [CoolProp](https://coolprop.org), the open-source equation-of-state library. CoolProp is
compiled from vendored source inside your normal build and linked into your executable. There is no
CMake, no Python, no download at build time, and no dynamic library to ship or to find at run time.

## Features

- **One command.** `cargo build` compiles CoolProp 8.0.0 together with the rest of your crates.
  The only tool needed is a C++17 compiler, which Rust needs on every platform anyway.
- **One file.** CoolProp is linked statically, so the executable runs without a `.dylib`, `.so` or
  `.dll` next to it and without a library path baked in.
- **Reference-quality data.** Helmholtz equations of state (backend `HEOS`) for fluids such as
  nitrogen, carbon dioxide, methane, hydrogen, helium, water and air, addressed by CoolProp's names.
- **Four input pairs, fourteen outputs**, all in SI units: pressure and temperature, enthalpy and
  pressure, pressure and quality, quality and temperature. Quality gives the saturation line and the
  two-phase region.
- **Thread safe.** Each state is owned by one thread, any number of states run in parallel, and the
  results are bit-identical to a single-threaded run.
- **Honest errors.** A failed call returns CoolProp's own message as a `String`; an output that is
  not a finite number is refused.
- **Checked against CMake.** On a grid of 2026 states of six fluids and in a full seal-simulation
  workload, every result is bit-identical to a CoolProp 8.0.0 built with CMake from the same commit.

## Installation

Add Gluid to `Cargo.toml`:

```toml
[dependencies]
gluid = { git = "https://github.com/GordonGu1998/Gluid", tag = "v0.1.0" }
```

The first build compiles about 66 C++ files: roughly 30 seconds on 11 cores and a few minutes on
one core. Later builds reuse the result.

| Platform | Needs |
|---|---|
| macOS | Xcode command line tools (`xcode-select --install`) |
| Windows | Visual Studio Build Tools with the C++ workload (the MSVC toolchain of Rust) |
| Linux | `g++` or `clang++` with the C++ standard library |

## Quick start

```rust
use gluid::{AbstractState, Output, Pair};

// Create the state of a pure fluid by the name CoolProp knows it by.
let mut nitrogen = AbstractState::new("Nitrogen").unwrap();

// Update it from two inputs in the order of the pair: pressure in Pa, temperature in K.
nitrogen.set_state(Pair::PressureTemperature, 2.0e6, 300.0).unwrap();

// Read any output of that state.
let density = nitrogen.get_output(Output::Density).unwrap();
let conductivity = nitrogen.get_output(Output::Conductivity).unwrap();
assert!((density - 22.52341).abs() < 1e-4); // kg/m3
assert!((conductivity - 0.02670885).abs() < 1e-7); // W/m/K
```

Saturation and two-phase states use the quality (0 for saturated liquid, 1 for saturated vapor):

```rust
use gluid::{AbstractState, Output, Pair};

let mut co2 = AbstractState::new("CO2").unwrap();
co2.set_state(Pair::PressureQuality, 5.0e6, 0.0).unwrap();
let saturation_temperature = co2.get_output(Output::Temperature).unwrap();
assert!((saturation_temperature - 287.4339).abs() < 1e-3); // K
```

An unknown fluid or an impossible state is an `Err` holding the message of CoolProp:

```rust
use gluid::AbstractState;

assert!(AbstractState::new("NoSuchFluid").is_err());
```

A state is cloned by replaying its last update, which gives every thread its own copy:

```rust
use gluid::{AbstractState, Output, Pair};
use std::thread;

let mut state = AbstractState::new("Methane").unwrap();
state.set_state(Pair::PressureTemperature, 1.0e6, 300.0).unwrap();
let handles: Vec<_> = (0..4)
    .map(|_| {
        let mut copy = state.clone();
        thread::spawn(move || copy.get_output(Output::Density).unwrap())
    })
    .collect();
for handle in handles {
    assert!(handle.join().unwrap() > 0.0);
}
```

Run `cargo run --example properties` to print a table.

## API overview

| Item | Purpose |
|---|---|
| `AbstractState::new(fluid)` | Create the state of a pure fluid (backend `HEOS`) |
| `AbstractState::set_state(pair, a, b)` | Update from two inputs in the order of the pair |
| `AbstractState::get_output(output)` | Read an output of the current state |
| `AbstractState::clone()` | New state with the same last update |
| `get_fluid_json(fluid)` | JSON description of a fluid (`None` if unknown) |
| `COOLPROP_VERSION` | Version of the linked CoolProp (`"8.0.0"`) |

| `Pair` | Inputs, in this order |
|---|---|
| `PressureTemperature` | pressure in Pa, temperature in K |
| `EnthalpyPressure` | specific enthalpy in J/kg, pressure in Pa |
| `PressureQuality` | pressure in Pa, quality 0 to 1 |
| `QualityTemperature` | quality 0 to 1, temperature in K |

| `Output` | Unit |
|---|---|
| `Density` | kg/m3 |
| `Enthalpy` | J/kg |
| `SpecificHeat` (at constant pressure) | J/kg/K |
| `Viscosity` | Pa s |
| `Conductivity` | W/m/K |
| `Temperature`, `CriticalTemperature`, `MinTemperature`, `TripleTemperature` | K |
| `Pressure`, `CriticalPressure`, `TriplePressure` | Pa |
| `IsothermalCompressibility` | 1/Pa |
| `IsobaricExpansion` | 1/K |

An output that was an input of the last update is returned exactly as it was given.

## Platform status

| Platform | Status |
|---|---|
| macOS arm64 | tested |
| macOS x86_64 | tested (Rosetta 2) |
| Windows x64 (MSVC) | tested by CI |
| Linux x64 (GCC) | tested by CI |
| Windows and Linux on ARM | not tested yet |

## Limitations

- Pure and pseudo-pure fluids with the backend `HEOS` only: no mixtures, no humid air, no
  incompressible liquids, no tabular backends, and no REFPROP (its library is proprietary).
- Four input pairs and the fourteen outputs above. Partial derivatives other than the two
  compressibility outputs are not exposed yet.
- SI units as plain `f64`; there are no unit types.

## Build environment

| Variable | Effect |
|---|---|
| `GLUID_CHECK_ONLY=1` | Skip the C++ build, for `cargo check` and `cargo clippy` of a target this machine cannot compile for. Nothing links. `DOCS_RS` does the same. |
| `MACOSX_DEPLOYMENT_TARGET` | Lowest macOS the objects run on (`.cargo/config.toml` of this repository sets 12.0). |
| `CXX`, `CXXFLAGS` | Compiler and flags, as for every crate that uses the `cc` crate. |

On Windows, `-C target-feature=+crt-static` selects the static C runtime for CoolProp as well.

## Updating CoolProp

`vendor/` and `data/` come from a pinned CoolProp commit through `tools/vendor.py`. The tool lets
CoolProp's own CMake write the compile database and the generated data, copies the files the
compiler reads, and records the release in `vendor/SOURCE`. CMake and Python 3 are needed for that
tool only. Change `TAG` and `COMMIT` in the script, run it, then run `cargo test`.

## Contributing

Issues and pull requests are welcome. Before a pull request run:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

New code follows the layout of the existing files: section dividers, a header for every function,
one imperative sentence per comment, and lines of at most 100 characters.

## License

Gluid is licensed under the [MIT License](LICENSE). The sources in `vendor/` and the data in
`data/` are not modified and keep the licenses of their projects (CoolProp MIT, Eigen MPL-2.0,
Boost and msgpack-c BSL-1.0, valijson BSD-2-Clause, incbin Unlicense, and others); see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Acknowledgements

Gluid is built on [CoolProp](https://coolprop.org) by Ian H. Bell and the CoolProp developers and is
not affiliated with the CoolProp project. If you publish results obtained with it, please cite:

> I. H. Bell, J. Wronski, S. Quoilin, V. Lemort, "Pure and Pseudo-pure Fluid Thermophysical Property
> Evaluation and the Open-Source Thermophysical Property Library CoolProp", *Industrial &
> Engineering Chemistry Research* 53 (2014) 2498-2508, https://doi.org/10.1021/ie4033999
