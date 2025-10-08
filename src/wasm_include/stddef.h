/* Minimal stddef.h for WASM compilation */

#ifndef _WASM_STDDEF_H
#define _WASM_STDDEF_H

#ifdef __cplusplus
extern "C" {
#endif

/* Basic types */
typedef unsigned long size_t;
typedef long ptrdiff_t;

#ifndef NULL
#define NULL ((void*)0)
#endif

#define offsetof(type, member) __builtin_offsetof(type, member)

#ifdef __cplusplus
}
#endif

#endif /* _WASM_STDDEF_H */
