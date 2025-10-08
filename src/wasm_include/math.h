/* Minimal math.h for WASM compilation
 * All functions are implemented in Rust using libm
 */

#ifndef _WASM_MATH_H
#define _WASM_MATH_H

#ifdef __cplusplus
extern "C" {
#endif

/* Float functions */
extern float floorf(float x);
extern float ceilf(float x);
extern float sqrtf(float x);
extern float sinf(float x);
extern float cosf(float x);
extern float expf(float x);
extern float logf(float x);
extern float powf(float x, float y);
extern float fabsf(float x);
extern long lrintf(float x);

/* Double functions */
extern double floor(double x);
extern double ceil(double x);
extern double sqrt(double x);
extern double sin(double x);
extern double cos(double x);
extern double exp(double x);
extern double log(double x);
extern double log10(double x);
extern double pow(double x, double y);
extern double fabs(double x);
extern long lrint(double x);

/* Constants */
#define M_PI 3.14159265358979323846

#ifdef __cplusplus
}
#endif

#endif /* _WASM_MATH_H */
