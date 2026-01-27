#![deny(rust_2018_idioms)]

#[cfg(feature = "generate_binding")]
use std::path::PathBuf;
use std::{env, fmt::Display, path::Path};

/// Outputs the library-file's prefix as word usable for actual arguments on
/// commands or paths.
const fn rustc_linking_word(is_static_link: bool) -> &'static str {
    if is_static_link {
        "static"
    } else {
        "dylib"
    }
}

/// Generates a new binding at `src/lib.rs` using `src/wrapper.h`.
#[cfg(feature = "generate_binding")]
fn generate_binding() {
    const ALLOW_UNCONVENTIONALS: &'static str = "#![allow(non_upper_case_globals)]\n\
                                                 #![allow(non_camel_case_types)]\n\
                                                 #![allow(non_snake_case)]\n";

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .raw_line(ALLOW_UNCONVENTIONALS)
        .generate()
        .expect("Unable to generate binding");

    let binding_target_path = PathBuf::new().join("src").join("lib.rs");

    bindings
        .write_to_file(binding_target_path)
        .expect("Could not write binding to the file at `src/lib.rs`");

    println!("cargo:info=Successfully generated binding.");
}

fn build_opus(is_static: bool) {
    let opus_path = Path::new("opus");

    println!(
        "cargo:info=Opus source path used: {:?}.",
        opus_path
            .canonicalize()
            .expect("Could not canonicalise to absolute path")
    );

    println!("cargo:info=Building Opus via CMake.");

    let mut opus_builder = cmake::Config::new(opus_path);

    opus_builder.profile("Release");

    if is_static {
        opus_builder.define("OPUS_STATIC_RUNTIME", "ON");
    }

    // cmake crate cannot handle cross-compile for Android (using cargo-ndk) well
    // so we need to manually set the toolchain file and other variables
    if is_cargo_ndk() {
        let ndk_path = find_android_ndk_path().expect("Could not find Android NDK");
        let cmake_toolchain_path = find_android_ndk_path()
            .map(|p| format!("{}/build/cmake/android.toolchain.cmake", p))
            .expect("Could not find Android NDK");
        let ninja_path = find_android_ninja().expect("Could not find ninja in Android SDK");
        let platform =
            env::var("CARGO_NDK_ANDROID_PLATFORM").expect("Could not find Android platform");

        let android_abi = env::var("CARGO_NDK_ANDROID_TARGET")
            .or_else(|_| env::var("ANDROID_ABI"))
            .expect("Could not find Android ABI");

        opus_builder
            .define("CMAKE_TOOLCHAIN_FILE", cmake_toolchain_path)
            .define("ANDROID_NDK", ndk_path)
            .define("ANDROID_ABI", android_abi)
            .define("ANDROID_PLATFORM", format!("android-{platform}"))
            .define("CMAKE_MAKE_PROGRAM", ninja_path)
            .define("CMAKE_SYSTEM_NAME", "Android")
            .generator("Ninja");
    }

    let opus_build_dir = opus_builder.build();
    link_opus(is_static, opus_build_dir.display())
}

fn link_opus(is_static: bool, opus_build_dir: impl Display) {
    let is_static_text = rustc_linking_word(is_static);

    println!(
        "cargo:info=Linking Opus as {} lib: {}",
        is_static_text, opus_build_dir
    );
    println!("cargo:rustc-link-lib={}=opus", is_static_text);
    println!("cargo:rustc-link-search=native={}/lib", opus_build_dir);
}

fn find_via_pkg_config(is_static: bool) -> bool {
    pkg_config::Config::new()
        .statik(is_static)
        .probe("opus")
        .is_ok()
}

/// Based on the OS or target environment we are building for,
/// this function will return an expected default library linking method.
///
/// If we build for Windows, MacOS, or Linux with musl, we will link statically.
/// However, if you build for Linux without musl, we will link dynamically.
///
/// **Info**:
/// This is a helper-function and may not be called if
/// if the `static`-feature is enabled, the environment variable
/// `LIBOPUS_STATIC` or `OPUS_STATIC` is set.
fn default_library_linking() -> bool {
    if cfg!(any(windows, target_os = "macos", target_env = "musl")) {
        true
    } else if cfg!(any(target_os = "freebsd", all(unix, target_env = "gnu"))) {
        false
    } else {
        unreachable!("Unsupported target environment.");
    }
}

fn find_installed_opus() -> Option<String> {
    if let Ok(lib_directory) = env::var("LIBOPUS_LIB_DIR") {
        Some(lib_directory)
    } else if let Ok(lib_directory) = env::var("OPUS_LIB_DIR") {
        Some(lib_directory)
    } else {
        None
    }
}

fn is_static_build() -> bool {
    if cfg!(feature = "static") && cfg!(feature = "dynamic") {
        default_library_linking()
    } else if cfg!(feature = "static")
        || env::var("LIBOPUS_STATIC").is_ok()
        || env::var("OPUS_STATIC").is_ok()
    {
        println!("cargo:info=Static feature or environment variable found.");

        true
    } else if cfg!(feature = "dynamic") {
        println!("cargo:info=Dynamic feature enabled.");

        false
    } else {
        println!("cargo:info=No feature or environment variable found, linking by default.");

        default_library_linking()
    }
}

fn is_cargo_ndk() -> bool {
    // cargo-ndk sets this variable so we use it to detect if we are cross-compiling for android
    env::var("CARGO_NDK_ANDROID_PLATFORM").is_ok()
}

fn find_android_sdk_path() -> Option<String> {
    if let Ok(sdk_path) = env::var("ANDROID_HOME") {
        Some(sdk_path)
    } else if let Ok(sdk_path) = env::var("ANDROID_SDK_HOME") {
        Some(sdk_path)
    } else {
        None
    }
}

fn find_android_ndk_path() -> Option<String> {
    if let Ok(ndk_path) = env::var("ANDROID_NDK_HOME") {
        Some(ndk_path)
    } else if let Ok(ndk_path) = env::var("ANDROID_NDK_ROOT") {
        Some(ndk_path)
    } else if let Some(sdk_path) = find_android_sdk_path() {
        let sdk_path = Path::new(&sdk_path);
        sdk_path.join("ndk").read_dir().ok().and_then(|entries| {
            entries
                .filter_map(|entry| {
                    entry.ok().and_then(|entry| {
                        if entry.file_type().ok()?.is_dir() {
                            entry.file_name().into_string().ok()
                        } else {
                            None
                        }
                    })
                })
                .next()
                .map(|ndk_version| {
                    sdk_path
                        .join("ndk")
                        .join(ndk_version)
                        .to_string_lossy()
                        .to_string()
                })
        })
    } else {
        None
    }
}

// find ninja executable in android sdk
fn find_android_ninja() -> Option<String> {
    if let Some(sdk_path) = find_android_sdk_path() {
        let sdk_path = Path::new(&sdk_path);
        sdk_path.join("cmake").read_dir().ok().and_then(|entries| {
            entries
                .filter_map(|entry| {
                    entry.ok().and_then(|entry| {
                        if entry.file_type().ok()?.is_dir() {
                            entry.file_name().into_string().ok()
                        } else {
                            None
                        }
                    })
                })
                .next()
                .map(|cmake_version| {
                    sdk_path
                        .join("cmake")
                        .join(cmake_version)
                        .join("bin")
                        .join("ninja")
                        .to_string_lossy()
                        .to_string()
                })
        })
    } else {
        // check if the `ninja` executable is in the PATH
        if let Ok(path) = env::var("PATH") {
            for entry in path.split(':') {
                let entry_path = Path::new(entry).join("ninja");
                if entry_path.exists() {
                    return Some(entry_path.to_string_lossy().to_string());
                }
            }
        }
        None
    }
}

fn main() {
    #[cfg(feature = "generate_binding")]
    generate_binding();

    let target = env::var("TARGET").unwrap();

    // Detect WASM target
    if target.starts_with("wasm32") {
        println!("cargo:warning=Building for WASM target: {}", target);
        build_opus_wasm_decoder();
        return;
    }

    let is_static = is_static_build();

    if cfg!(any(unix, target_env = "gnu")) {
        if std::env::var("LIBOPUS_NO_PKG").is_ok() || std::env::var("OPUS_NO_PKG").is_ok() {
            println!("cargo:info=Bypassed `pkg-config`.");
        } else if find_via_pkg_config(is_static) {
            println!("cargo:info=Found `Opus` via `pkg_config`.");

            return;
        } else {
            println!("cargo:info=`pkg_config` could not find `Opus`.");
        }
    }

    if let Some(installed_opus) = find_installed_opus() {
        link_opus(is_static, installed_opus);
    } else {
        build_opus(is_static);
    }
}

/// Build Opus for WASM target
///
/// Use cc crate to directly compile C source files, avoiding CMake dependencies.
/// Automatically obtain source file list by parsing official Opus .mk files.
fn build_opus_wasm_decoder() {
    println!("cargo:info=Building Opus (full: encoder+decoder) for WASM using cc crate");

    let mut build = cc::Build::new();

    // === Basic configuration ===
    build
        .target("wasm32-unknown-unknown")
        .opt_level(2)
        .include("opus/include")
        .include("opus/celt")
        .include("opus/silk")
        .include("opus/silk/float")
        .include("src/wasm_include");

    // === Define necessary macros ===
    build
        .define("OPUS_BUILD", None)
        .define("USE_ALLOCA", None)
        .define("HAVE_LRINT", None)
        .define("HAVE_LRINTF", None)
        .define("OVERRIDE_OPUS_ALLOC", None)
        .define("OVERRIDE_OPUS_FREE", None)
        .define("OVERRIDE_OPUS_REALLOC", None)
        .define("OVERRIDE_OPUS_ALLOC_SCRATCH", None)
        .define("NONTHREADSAFE_PSEUDOSTACK", None)
        .define("OPUS_EXPORT=", None)
        .define("VAR_ARRAYS", None)
        // Floating point configuration (mutually exclusive with FIXED_POINT)
        .define("FLOATING_POINT", None);

    // Force include custom memory allocation header file
    build.flag("-include./src/wasm_include/opus_alloc.h");

    // === Parse official .mk files to get source files ===
    println!("cargo:info=Parsing Opus source lists from .mk files");

    let opus_sources = parse_makefile_sources(
        "opus/opus_sources.mk",
        &["OPUS_SOURCES", "OPUS_SOURCES_FLOAT"],
    );

    let celt_sources = parse_makefile_sources("opus/celt_sources.mk", &["CELT_SOURCES"]);

    let silk_sources = parse_makefile_sources(
        "opus/silk_sources.mk",
        &["SILK_SOURCES", "SILK_SOURCES_FLOAT"],
    );

    // Filter out non-WASM-compatible source files
    let all_sources: Vec<String> = opus_sources
        .into_iter()
        .chain(celt_sources)
        .chain(silk_sources)
        .filter(|s| is_wasm_compatible(s))
        .collect();

    println!(
        "cargo:info=Found {} source files for WASM",
        all_sources.len()
    );

    // Add source files to build
    for source in &all_sources {
        let full_path = format!("opus/{}", source);
        build.file(&full_path);
    }

    println!("cargo:info=Compiling Opus library for WASM");
    build.compile("opus");

    println!("cargo:info=Successfully built Opus (full) for WASM");
}

/// Parse Makefile (.mk) files, extract source file list
///
/// # Parameters
/// - `mk_file`: .mk file path
/// - `variable_names`: List of variable names to extract (e.g., "OPUS_SOURCES", "OPUS_SOURCES_FLOAT")
///
/// # Returns
/// List of source file paths (relative to opus/ directory)
fn parse_makefile_sources(mk_file: &str, variable_names: &[&str]) -> Vec<String> {
    let content = std::fs::read_to_string(mk_file)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", mk_file, e));

    let mut sources = Vec::new();
    let mut in_target_var = false;
    let mut current_parsing_var: Option<String> = None;
    let mut var_file_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Check if it's a variable definition (contains =)
        if line.contains('=') {
            // Get variable name
            let var_name = line.split('=').next().unwrap().trim();

            // Check if it's one of our target variables
            if variable_names.contains(&var_name) {
                in_target_var = true;
                current_parsing_var = Some(var_name.to_string());
                println!("cargo:info=    Start parsing variable: {}", var_name);

                // Check if there are files in the same line
                if let Some(after_eq) = line.split('=').nth(1) {
                    let after_eq = after_eq.trim();
                    if after_eq.ends_with(".c \\") {
                        let file = after_eq.trim_end_matches(" \\").trim();
                        sources.push(file.to_string());
                        *var_file_counts.entry(var_name.to_string()).or_insert(0) += 1;
                    } else if after_eq.ends_with(".c") && !after_eq.is_empty() {
                        sources.push(after_eq.trim().to_string());
                        *var_file_counts.entry(var_name.to_string()).or_insert(0) += 1;
                        in_target_var = false; // Variable definition ends (last file without \)
                        if let Some(ref vname) = current_parsing_var {
                            println!(
                                "cargo:info=    {} parsing completed: {} files",
                                vname,
                                var_file_counts.get(vname).unwrap_or(&0)
                            );
                        }
                        current_parsing_var = None;
                    }
                }
            } else {
                // This is another variable, exit parsing of current target variable
                if in_target_var {
                    if let Some(ref vname) = current_parsing_var {
                        println!(
                            "cargo:info=    {} parsing completed: {} files",
                            vname,
                            var_file_counts.get(vname).unwrap_or(&0)
                        );
                    }
                }
                in_target_var = false;
                current_parsing_var = None;
            }
            continue;
        }

        // If in target variable definition, extract filename
        if in_target_var {
            if line.ends_with(".c \\") {
                let file = line.trim_end_matches(" \\").trim();
                if !file.is_empty() {
                    sources.push(file.to_string());
                    if let Some(ref vname) = current_parsing_var {
                        *var_file_counts.entry(vname.clone()).or_insert(0) += 1;
                    }
                }
            } else if line.ends_with(".c") {
                let file = line.trim();
                if !file.is_empty() {
                    sources.push(file.to_string());
                    if let Some(ref vname) = current_parsing_var {
                        *var_file_counts.entry(vname.clone()).or_insert(0) += 1;
                    }
                }
                // Variable definition ends (last file without \)
                if let Some(ref vname) = current_parsing_var {
                    println!(
                        "cargo:info=    {} parsing completed: {} files",
                        vname,
                        var_file_counts.get(vname).unwrap_or(&0)
                    );
                }
                in_target_var = false;
                current_parsing_var = None;
            }
        }
    }
    println!("cargo:info=  {} - found {} files", mk_file, sources.len());
    sources
}

/// Determine if source file is WASM compatible
///
/// Filter out:
/// - Platform-specific code (x86, arm, mips)
/// - Assembly files (.s, .s.in)
fn is_wasm_compatible(source: &str) -> bool {
    // exclude platform-specific directories
    if source.contains("/x86/") || source.contains("/arm/") || source.contains("/mips/") {
        return false;
    }

    // exclude assembly files
    if source.ends_with(".s") || source.ends_with(".s.in") {
        return false;
    }

    true
}
