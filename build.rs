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
        let android_abi = env::var("CARGO_NDK_ANDROID_TARGET").expect("Could not find Android ABI");

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
