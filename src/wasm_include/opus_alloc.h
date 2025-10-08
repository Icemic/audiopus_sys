/* WASM overrides for Opus memory functions
 * These functions are implemented in Rust
 */

#ifndef _WASM_OPUS_ALLOC_H
#define _WASM_OPUS_ALLOC_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opus memory management functions - implemented in Rust */
extern void *opus_alloc(size_t size);
extern void *opus_realloc(void *ptr, size_t size);
extern void opus_free(void *ptr);
extern void *opus_alloc_scratch(size_t size);

#ifdef __cplusplus
}
#endif

#endif /* _WASM_OPUS_ALLOC_H */
