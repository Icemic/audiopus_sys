mod bindings;
// wasm memory allocator module
// Provides unified memory management for Opus C code
#[cfg(target_arch = "wasm32")]
mod wasm_alloc;
// wasm libm module
// Provides math functions for WebAssembly targets
#[cfg(target_arch = "wasm32")]
mod wasm_libm;

pub use bindings::*;
