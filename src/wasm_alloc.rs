//! WASM memory allocator
//!
//! Provides unified memory management for Opus C code.
//! Currently uses Rust's standard global allocator (dlmalloc by default on wasm32).
//!
//! Can be easily replaced with lol_alloc or other WASM-optimized allocators in the future.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::sync::Mutex;

/// Allocation record table, used to track the size of each allocation
///
/// This is necessary because C's free() does not pass the size parameter,
/// but Rust's dealloc requires the original Layout.
static ALLOC_TABLE: Mutex<Option<HashMap<usize, Layout>>> = Mutex::new(None);

/// Initialize the allocation record table (lazy loading)
fn init_alloc_table() {
    let mut table = ALLOC_TABLE.lock().unwrap();
    if table.is_none() {
        *table = Some(HashMap::new());
    }
}

/// Opus memory allocation function
///
/// Overrides opus_alloc(), uses Rust's global allocator.
///
/// # Safety
///
/// This is an FFI function, the caller must ensure:
/// - size is a reasonable value
/// - the returned pointer will be freed via opus_free
#[no_mangle]
pub unsafe extern "C" fn opus_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }

    // Use 8-byte alignment (suitable for most platforms)
    let layout = match Layout::from_size_align(size, 8) {
        Ok(layout) => layout,
        Err(_) => {
            eprintln!("opus_alloc: invalid size/align: {}", size);
            return std::ptr::null_mut();
        }
    };

    let ptr = alloc(layout);

    if ptr.is_null() {
        eprintln!("opus_alloc: allocation failed for size {}", size);
        return ptr;
    }

    // Record allocation info
    init_alloc_table();
    let mut table = ALLOC_TABLE.lock().unwrap();
    if let Some(ref mut map) = *table {
        map.insert(ptr as usize, layout);
    }

    ptr
}

/// Opus memory free function
///
/// Overrides opus_free(), frees memory allocated by opus_alloc.
///
/// # Safety
///
/// The caller must ensure:
/// - ptr was allocated by opus_alloc or opus_realloc
/// - ptr will only be freed once
#[no_mangle]
pub unsafe extern "C" fn opus_free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    init_alloc_table();
    let mut table = ALLOC_TABLE.lock().unwrap();

    if let Some(ref mut map) = *table {
        if let Some(layout) = map.remove(&(ptr as usize)) {
            dealloc(ptr, layout);
        } else {
            eprintln!("opus_free: attempting to free untracked pointer: {:p}", ptr);
            // Note: This may be a bug, but for safety, we do not panic
        }
    }
}

/// Opus memory reallocation function
///
/// Overrides opus_realloc(), resizes previously allocated memory.
///
/// # Safety
///
/// The caller must ensure:
/// - ptr was allocated by opus_alloc, or is null
/// - the original pointer is not used after this call
#[no_mangle]
pub unsafe extern "C" fn opus_realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
    // Special case: if ptr is null, equivalent to opus_alloc
    if ptr.is_null() {
        return opus_alloc(new_size);
    }

    // Special case: if new_size is 0, equivalent to opus_free
    if new_size == 0 {
        opus_free(ptr);
        return std::ptr::null_mut();
    }

    init_alloc_table();
    let mut table = ALLOC_TABLE.lock().unwrap();

    if let Some(ref mut map) = *table {
        if let Some(&old_layout) = map.get(&(ptr as usize)) {
            let new_layout = match Layout::from_size_align(new_size, 8) {
                Ok(layout) => layout,
                Err(_) => {
                    eprintln!("opus_realloc: invalid new size/align: {}", new_size);
                    return std::ptr::null_mut();
                }
            };

            // Manual realloc: allocate new memory, copy data, free old memory
            let new_ptr = alloc(new_layout);
            if new_ptr.is_null() {
                eprintln!("opus_realloc: reallocation failed for size {}", new_size);
                return new_ptr;
            }

            // Copy data (copy the smaller size)
            let copy_size = old_layout.size().min(new_size);
            std::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);

            // Free old memory
            dealloc(ptr, old_layout);

            // Update record
            map.remove(&(ptr as usize));
            map.insert(new_ptr as usize, new_layout);

            new_ptr
        } else {
            eprintln!(
                "opus_realloc: attempting to realloc untracked pointer: {:p}",
                ptr
            );
            // Try to allocate new memory as a fallback
            let new_ptr = opus_alloc(new_size);
            if !new_ptr.is_null() {
                // Cannot determine original size, can only copy new_size bytes (may be unsafe)
                std::ptr::copy_nonoverlapping(ptr, new_ptr, new_size);
            }
            new_ptr
        }
    } else {
        std::ptr::null_mut()
    }
}

/// Opus scratch memory allocation function
///
/// Overrides opus_alloc_scratch(), used for temporary buffers.
/// Currently implemented the same as opus_alloc, can be optimized to stack allocation or memory pool in the future.
#[no_mangle]
pub unsafe extern "C" fn opus_alloc_scratch(size: usize) -> *mut u8 {
    // Current simple implementation: directly use opus_alloc
    // Future optimizations:
    // 1. Use bump allocator
    // 2. Use thread-local memory pool
    // 3. Use stack allocation (if size is small enough)
    opus_alloc(size)
}

// Optional: Provide wrappers for code that may directly call malloc/free
// These functions are only exported under the WASM target

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    opus_alloc(size)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8) {
    opus_free(ptr)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let total = match nmemb.checked_mul(size) {
        Some(total) => total,
        None => {
            eprintln!("calloc: overflow in size calculation: {} * {}", nmemb, size);
            return std::ptr::null_mut();
        }
    };

    if total == 0 {
        return std::ptr::null_mut();
    }

    let ptr = opus_alloc(total);
    if !ptr.is_null() {
        // calloc needs to zero the memory
        std::ptr::write_bytes(ptr, 0, total);
    }
    ptr
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    opus_realloc(ptr, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_free() {
        unsafe {
            let ptr = opus_alloc(100);
            assert!(!ptr.is_null());
            opus_free(ptr);
        }
    }

    #[test]
    fn test_alloc_zero() {
        unsafe {
            let ptr = opus_alloc(0);
            assert!(ptr.is_null());
        }
    }

    #[test]
    fn test_realloc() {
        unsafe {
            let ptr = opus_alloc(100);
            assert!(!ptr.is_null());

            let ptr2 = opus_realloc(ptr, 200);
            assert!(!ptr2.is_null());

            opus_free(ptr2);
        }
    }

    #[test]
    fn test_calloc() {
        unsafe {
            let ptr = calloc(10, 10);
            assert!(!ptr.is_null());

            // Verify memory is zeroed
            for i in 0..100 {
                assert_eq!(*ptr.add(i), 0);
            }

            opus_free(ptr);
        }
    }
}
