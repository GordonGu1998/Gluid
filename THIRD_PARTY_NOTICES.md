# Third-Party Notices

`vendor` and `data` hold unmodified files of the projects below. Each folder
carries the license texts of its project. `vendor/SOURCE` names the CoolProp release.

| Project | Folder in `vendor` | License |
|---|---|---|
| CoolProp 8.0.0 (Ian H. Bell and other CoolProp developers), with its fluid data in `data` | `coolprop` | MIT |
| miniz 3.1.1 | `coolprop/externals/miniz-3.1.1` | MIT |
| incbin | `coolprop/externals/incbin` | Unlicense |
| Eigen 5.0.1 | `eigen` | MPL-2.0 per file, some files BSD or Apache-2.0 (see `COPYING.*`) |
| {fmt} | `fmt` | MIT |
| nlohmann/json | `json` | MIT |
| valijson | `valijson` | BSD-2-Clause |
| msgpack-c | `msgpack` | BSL-1.0 |
| Boost headers | `boost` | BSL-1.0 |
| IF97 (Ian H. Bell) | `if97` | MIT |
| REFPROP headers (declarations only, the REFPROP backend is not used) | `refprop_headers` | MIT |

Eigen is not modified, so only its notice is owed. `multicomplex`, which CoolProp fetches for its
tests and which has no license file, is not included.
