//==================================================================================================
// Import
//==================================================================================================
// Import Components from the Standard Library======================================================
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

//==================================================================================================
// Constants
//==================================================================================================
// Define the Headers Made from the JSON Files with the Input File and the Variable=================
const TEXT_HEADERS: [(&str, &str, &str); 9] = [
    (
        "all_incompressibles.json",
        "all_incompressibles_JSON.h",
        "all_incompressibles_JSON",
    ),
    (
        "mixtures/mixture_departure_functions.json",
        "mixture_departure_functions_JSON.h",
        "mixture_departure_functions_JSON",
    ),
    (
        "mixtures/mixture_binary_pairs.json",
        "mixture_binary_pairs_JSON.h",
        "mixture_binary_pairs_JSON",
    ),
    (
        "mixtures/predefined_mixtures.json",
        "predefined_mixtures_JSON.h",
        "predefined_mixtures_JSON",
    ),
    (
        "cubics/all_cubic_fluids.json",
        "all_cubics_JSON.h",
        "all_cubics_JSON",
    ),
    (
        "cubics/cubic_fluids_schema.json",
        "cubic_fluids_schema_JSON.h",
        "cubic_fluids_schema_JSON",
    ),
    (
        "pcsaft/pcsaft_fluids_schema.json",
        "pcsaft_fluids_schema_JSON.h",
        "pcsaft_fluids_schema_JSON",
    ),
    (
        "pcsaft/all_pcsaft_fluids.json",
        "all_pcsaft_JSON.h",
        "all_pcsaft_JSON",
    ),
    (
        "pcsaft/mixture_binary_pairs_pcsaft.json",
        "mixture_binary_pairs_pcsaft_JSON.h",
        "mixture_binary_pairs_pcsaft_JSON",
    ),
];

// Define the Include Folders under vendor in the Order of the CMake Build of CoolProp==============
const INCLUDE: [&str; 13] = [
    "boost",
    "coolprop",
    "eigen",
    "msgpack",
    "coolprop/externals/miniz-3.1.1",
    "coolprop/externals/incbin",
    "fmt",
    "json",
    "valijson",
    "if97",
    "refprop_headers",
    "coolprop/include",
    "coolprop/src",
];

//==================================================================================================
// Function Name: get_hex
//   Description: Write the bytes as hexadecimal numbers with sixteen of them on a line
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn get_hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 6);
    for (i, line) in bytes.chunks(16).enumerate() {
        if i > 0 {
            text.push_str(",\n");
        }
        for (j, byte) in line.iter().enumerate() {
            if j > 0 {
                text.push_str(", ");
            }
            write!(text, "0x{byte:02x}").expect("a string takes the text");
        }
    }
    text
}

//==================================================================================================
// Function Name: write_if_changed
//   Description: Write the file unless it holds the text already so that cargo sees no change
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn write_if_changed(path: &Path, text: &str) {
    if fs::read_to_string(path).is_ok_and(|old| old == text) {
        return;
    }
    fs::write(path, text).expect("the generated header is written");
}

//==================================================================================================
// Function Name: write_headers
//   Description: Write the headers CoolProp makes with a Python script from the data files
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn write_headers(data: &Path, folder: &Path, version: &str, commit: &str, is_header: bool) {
    // Write the Headers Holding a JSON File as Text with a Terminating Zero========================
    fs::create_dir_all(folder).expect("the folder of the headers is created");
    for (input, output, variable) in TEXT_HEADERS {
        let mut bytes = fs::read(data.join(input)).expect("the data file is read");
        bytes.push(0);
        let text = format!(
            "// JSON file encoded in binary form\nstatic constexpr char {variable}_binary[] = \
             {{\n{}\n}};\n\n// Combined into a single std::string \nstatic constexpr \
             std::string_view {variable}(std::begin({variable}_binary), \
             std::size({variable}_binary));",
            get_hex(&bytes)
        );
        write_if_changed(&folder.join(output), &text);
    }

    // Write the Headers of the Version and of the Source Revision==================================
    let text = format!("\nstatic char version [] =\"{version}\";");
    write_if_changed(&folder.join("cpversion.h"), &text);
    let text = format!("\nstd::string gitrevision = \"{commit}\";");
    write_if_changed(&folder.join("gitrevision.h"), &text);

    // Write the Fluids as Binary Data for the Compilers Reading Them from a Header=================
    if is_header {
        let bytes = fs::read(data.join("all_fluids.cbor")).expect("the fluid data is read");
        let text = format!(
            "#ifdef __cplusplus\nextern \"C\" {{\n#endif\n\nINCBIN_CONST INCBIN_ALIGN unsigned \
             char gall_fluids_CBORData[] = {{\n{}\n}};\nINCBIN_CONST INCBIN_ALIGN unsigned char \
             *const gall_fluids_CBOREnd = gall_fluids_CBORData + \
             sizeof(gall_fluids_CBORData);\nINCBIN_CONST unsigned int gall_fluids_CBORSize = \
             sizeof(gall_fluids_CBORData);\n\n#ifdef __cplusplus\n}}\n#endif\n",
            get_hex(&bytes)
        );
        write_if_changed(&folder.join("all_fluids_CBOR.h"), &text);
    }
}

//==================================================================================================
// Function Name: get_sources
//   Description: Collect the C++ files below a folder in a fixed order
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn get_sources(folder: &Path, list: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(folder)
        .expect("the source folder is read")
        .map(|entry| entry.expect("the entry is read").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            get_sources(&path, list);
        } else if path.extension().is_some_and(|extension| extension == "cpp") {
            list.push(path);
        }
    }
}

//==================================================================================================
// Function Name: main
//   Description: Write the generated headers and compile CoolProp into a static library
//     Edit Date: Oct 18th, 2026.
//==================================================================================================
fn main() {
    // Rebuild When the Script the Source or the Data Changes=======================================
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=data");

    // Find the Folders and Read the Version of the Source==========================================
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets the out dir"));
    let vendor = root.join("vendor");
    let text = fs::read_to_string(vendor.join("SOURCE")).expect("the version is read");
    let mut lines = text.lines();
    let version = lines.next().expect("the version file has a version");
    let commit = lines.next().expect("the version file has a revision");
    println!("cargo:rustc-env=GLUID_COOLPROP_VERSION={version}");

    // Skip the Compiler When Only the Code Is Checked for a Target This Machine Cannot Build=======
    println!("cargo:rerun-if-env-changed=GLUID_CHECK_ONLY");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var_os("GLUID_CHECK_ONLY").is_some() || env::var_os("DOCS_RS").is_some() {
        println!("cargo:warning=gluid: CoolProp is not compiled so nothing can be linked");
        return;
    }

    // Set Up the Compiler for C++17 with the Definitions of the CMake Build========================
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .warnings(false)
        .debug(false)
        .opt_level(3);
    build
        .define("NDEBUG", None)
        .define("EXTERNC", None)
        .define("VALIJSON_USE_EXCEPTIONS", "1");
    let is_msvc = build.get_compiler().is_like_msvc();
    if is_msvc {
        build
            .flag("/bigobj")
            .flag("/utf-8")
            .define("_CRT_SECURE_NO_WARNINGS", None);
    }

    // Write the Headers and Add the Include Folders================================================
    let generated = out.join("generated");
    write_headers(&root.join("data"), &generated, version, commit, is_msvc);
    for folder in INCLUDE {
        build.include(vendor.join(folder));
    }
    build.include(&generated);

    // Let the Assembler of the Other Compilers Find the Fluid Data=================================
    build.include(root.join("data"));

    // Compile the Sources of CoolProp==============================================================
    let mut sources = Vec::new();
    get_sources(&vendor.join("coolprop/src"), &mut sources);
    build.files(&sources);
    build.compile("gluid_coolprop");

    // Compile the Zip Library of CoolProp Written in C=============================================
    let mut zip = cc::Build::new();
    zip.warnings(false)
        .debug(false)
        .opt_level(3)
        .define("NDEBUG", None);
    zip.file(vendor.join("coolprop/externals/miniz-3.1.1/miniz.c"));
    zip.compile("gluid_miniz");
}
