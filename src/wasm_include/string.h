/* Minimal string.h for WASM compilation
 * String functions are provided by compiler-builtins
 */

#ifndef _WASM_STRING_H
#define _WASM_STRING_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Memory functions - provided by compiler-builtins */
extern void *memcpy(void *dest, const void *src, size_t n);
extern void *memmove(void *dest, const void *src, size_t n);
extern void *memset(void *s, int c, size_t n);
extern int memcmp(const void *s1, const void *s2, size_t n);

/* String functions */
extern size_t strlen(const char *s);
extern char *strcpy(char *dest, const char *src);
extern char *strncpy(char *dest, const char *src, size_t n);
extern int strcmp(const char *s1, const char *s2);
extern int strncmp(const char *s1, const char *s2, size_t n);

#ifdef __cplusplus
}
#endif

#endif /* _WASM_STRING_H */
