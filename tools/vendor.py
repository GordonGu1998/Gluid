#!/usr/bin/env python3
"""Rebuild vendor and data from a pinned CoolProp checkout.

Maintainer tool only: CMake, Python 3 and a POSIX C++ compiler are needed here and never to build
the crates. CoolProp's CMake runs once to produce the compile database and the generated data; the
files the compiler reads are collected with `-M` and copied. Running the tool on another platform
only adds files, so the union of the platforms stays in the tree.

Usage: tools/vendor.py [--work DIR]
"""
import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

TAG = "v8.0.0"
COMMIT = "ae81610e7d23efc57f9d051c8e70a4d66e87537f"
REPOSITORY = "https://github.com/CoolProp/CoolProp.git"

ROOT = Path(__file__).resolve().parent.parent

# Files CoolProp's generate_headers.py writes into include/, rebuilt by build.rs instead
GENERATED = {
    "all_cubics_JSON.h", "all_fluids_CBOR.h", "all_incompressibles_JSON.h", "all_pcsaft_JSON.h",
    "cpversion.h", "cubic_fluids_schema_JSON.h", "gitrevision.h", "mixture_binary_pairs_JSON.h",
    "mixture_binary_pairs_pcsaft_JSON.h", "mixture_departure_functions_JSON.h",
    "pcsaft_fluids_schema_JSON.h", "predefined_mixtures_JSON.h",
}

# Data files read by build.rs, relative to CoolProp's dev folder
DATA = [
    "all_fluids.cbor", "all_incompressibles.json", "mixtures/mixture_departure_functions.json",
    "mixtures/mixture_binary_pairs.json", "mixtures/predefined_mixtures.json",
    "cubics/all_cubic_fluids.json", "cubics/cubic_fluids_schema.json",
    "pcsaft/pcsaft_fluids_schema.json", "pcsaft/all_pcsaft_fluids.json",
    "pcsaft/mixture_binary_pairs_pcsaft.json",
]

# Dependency, folder of the CPM cache, include root inside it, whole folders besides the closure,
# license files beside the include root. Whole folders hold what only other platforms read.
DEPENDENCIES = [
    ("boost", "boost_headers", "", [
        "boost/config", "boost/predef", "boost/core", "boost/detail", "boost/utility",
        "boost/assert", "boost/type_traits"], ["LICENSE"]),
    ("msgpack", "msgpack-c", "include", ["msgpack/predef"],
     ["COPYING", "LICENSE_1_0.txt", "NOTICE"]),
    ("fmt", "fmt", "include", ["fmt"], ["LICENSE"]),
    ("json", "nlohmann_json", "include", ["nlohmann"], ["LICENSE.MIT"]),
    ("valijson", "valijson", "include", ["valijson"], ["LICENSE"]),
    ("if97", "if97", "", [], ["LICENSE"]),
    ("refprop_headers", "refprop_headers", "", [], ["LICENSE"]),
    ("eigen", "eigen", "", ["Eigen/src/Core"],
     ["COPYING.APACHE", "COPYING.BSD", "COPYING.MINPACK", "COPYING.MPL2", "COPYING.README",
      "LICENSE"]),
]


def run(command, **kwargs):
    subprocess.run(command, check=True, **kwargs)


def get_checkout(work):
    checkout = (work / "coolprop").resolve()
    if not (checkout / ".git").exists():
        run(["git", "clone", "--branch", TAG, "--depth", "1", REPOSITORY, str(checkout)])
    head = subprocess.run(["git", "-C", str(checkout), "rev-parse", "HEAD"], check=True,
                          capture_output=True, text=True).stdout.strip()
    if head != COMMIT:
        sys.exit(f"error: {TAG} is at {head}, expected {COMMIT}")
    return checkout


def run_cmake(checkout, work):
    build = work / "cmake-build"
    env = dict(os.environ, CPM_SOURCE_CACHE=str(work / "cpm"))
    run(["cmake", "-S", str(checkout), "-B", str(build), "-DCMAKE_BUILD_TYPE=Release",
         "-DCOOLPROP_STATIC_LIBRARY=ON", "-DCOOLPROP_RELEASE=ON", "-DCOOLPROP_EXTERNC_LIBRARY=ON",
         "-DBUILD_TESTING=OFF", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
         "-DCMAKE_POLICY_VERSION_MINIMUM=3.5"], env=env)
    return build


def get_closure(build):
    closure = set()
    sources = []
    for entry in json.loads((build / "compile_commands.json").read_text()):
        words = shlex.split(entry["command"])
        command = []
        skip = False
        for word in words:
            if skip:
                skip = False
            elif word == "-o":
                skip = True
            elif word != "-c":
                command.append(word)
        sources.append(Path(entry["file"]).resolve())
        text = subprocess.run(command + ["-M"], check=True, capture_output=True, text=True,
                              cwd=entry["directory"]).stdout
        names = text.replace("\\\n", " ").split(": ", 1)[1].split()
        closure.update(Path(name).resolve() for name in names)
    return closure, sources


def copy_file(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)


def copy_tree(source, target, keep=lambda path: True):
    for path in sorted(source.rglob("*")):
        if path.is_file() and keep(path):
            copy_file(path, target / path.relative_to(source))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--work", default=str(ROOT / "target" / "vendor-work"))
    work = Path(parser.parse_args().work).resolve()
    work.mkdir(parents=True, exist_ok=True)

    checkout = get_checkout(work)
    build = run_cmake(checkout, work)
    closure, sources = get_closure(build)
    vendor = ROOT / "vendor"
    cache = {p.name: next(d for d in p.iterdir() if d.is_dir()).resolve()
             for p in (work / "cpm").iterdir() if p.is_dir()}

    # CoolProp: every source of the compile database, every header of include and src
    target = vendor / "coolprop"
    for source in sources:
        copy_file(source, target / source.relative_to(checkout))
    for folder in ("include", "src"):
        copy_tree(checkout / folder, target / folder,
                  lambda p: p.suffix in (".h", ".hpp") and p.name not in GENERATED
                  and "Tests" not in p.parts)
    copy_tree(checkout / "externals" / "miniz-3.1.1", target / "externals" / "miniz-3.1.1",
              lambda p: p.parent.name == "miniz-3.1.1"
              and (p.suffix in (".h", ".c") or p.name == "LICENSE"))
    copy_tree(checkout / "externals" / "incbin", target / "externals" / "incbin",
              lambda p: p.name in ("incbin.h", "UNLICENSE"))
    copy_file(checkout / "LICENSE", target / "LICENSE")
    # Not in an include folder: on a case-insensitive disk VERSION hides the C++20 header version
    (vendor / "SOURCE").write_text(f"{TAG.lstrip('v')}\n{COMMIT}\n")

    # Dependencies: the closure plus whole folders plus the license texts
    for name, folder, include, whole, licenses in DEPENDENCIES:
        base = cache[folder]
        root = base / include if include else base
        target = vendor / name
        for path in closure:
            if root in path.parents:
                copy_file(path, target / path.relative_to(root))
        for part in whole:
            copy_tree(root / part, target / part)
        for license in licenses:
            copy_file(base / license, target / license)
    eigen = cache["eigen"]
    copy_tree(eigen / "Eigen", vendor / "eigen" / "Eigen",
              lambda p: p.parent.name == "Eigen")
    copy_tree(eigen / "unsupported" / "Eigen" / "src" / "Polynomials",
              vendor / "eigen" / "unsupported" / "Eigen" / "src" / "Polynomials")
    copy_file(eigen / "unsupported" / "Eigen" / "Polynomials",
              vendor / "eigen" / "unsupported" / "Eigen" / "Polynomials")
    copy_file(eigen / "signature_of_eigen3_matrix_library",
              vendor / "eigen" / "signature_of_eigen3_matrix_library")

    # Data: the files the generated headers are made from
    for name in DATA:
        copy_file(checkout / "dev" / name, ROOT / "data" / name)


if __name__ == "__main__":
    main()
