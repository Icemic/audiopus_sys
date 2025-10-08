/* Minimal stdlib.h for WASM compilation
 * Memory functions are provided by Rust
 */

#ifndef _WASM_STDLIB_H
#define _WASM_STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Memory allocation - implemented in Rust */
extern void *malloc(size_t size);
extern void *calloc(size_t nmemb, size_t size);
extern void *realloc(void *ptr, size_t size);
extern void free(void *ptr);

/* Process control */
extern void abort(void) __attribute__((noreturn));
extern void exit(int status) __attribute__((noreturn));

/* Conversion functions */
extern int atoi(const char *nptr);
extern long atol(const char *nptr);
extern long long atoll(const char *nptr);

/* Absolute value */
extern int abs(int j);
extern long labs(long j);
extern long long llabs(long long j);

#ifdef __cplusplus
}
#endif

#endif /* _WASM_STDLIB_H */
