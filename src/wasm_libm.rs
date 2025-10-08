//! ============================================================================
//! Math function implementations (using libm)
//! Provide standard C math library functions for Opus C code
//! ============================================================================

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn lrintf(x: f32) -> isize {
    libm::roundf(x) as isize
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn lrint(x: f64) -> isize {
    libm::round(x) as isize
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn exp(x: f64) -> f64 {
    libm::exp(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn expf(x: f32) -> f32 {
    libm::expf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn log(x: f64) -> f64 {
    libm::log(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn logf(x: f32) -> f32 {
    libm::logf(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn log10(x: f64) -> f64 {
    libm::log10(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn log10f(x: f32) -> f32 {
    libm::log10f(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn fabs(x: f64) -> f64 {
    libm::fabs(x)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn fabsf(x: f32) -> f32 {
    libm::fabsf(x)
}
