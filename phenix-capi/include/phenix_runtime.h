#ifndef PHENIX_RUNTIME_H
#define PHENIX_RUNTIME_H

#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

int phenix_runtime_uint_encode(uint64_t value, FILE *stream);

int phenix_runtime_uint_encode_many(const uint64_t *values, size_t n, FILE *stream);

int phenix_runtime_sint_encode(int64_t value, FILE *stream);

int phenix_runtime_sint_encode_many(const int64_t *values, size_t n, FILE *stream);

int phenix_runtime_float_encode(double value, FILE *stream);

int phenix_runtime_float_encode_many(const double *values, size_t n, FILE *stream);

int phenix_runtime_bool_encode(bool value, FILE *stream);

int phenix_runtime_bool_encode_many(const bool *values, size_t n, FILE *stream);

int phenix_runtime_u8_encode(uint8_t value, FILE *stream);

int phenix_runtime_u8_encode_many(const uint8_t *values, size_t n, FILE *stream);

int phenix_runtime_u16_encode(uint16_t value, FILE *stream);

int phenix_runtime_u16_encode_many(const uint16_t *values, size_t n, FILE *stream);

int phenix_runtime_u32_encode(uint32_t value, FILE *stream);

int phenix_runtime_u32_encode_many(const uint32_t *values, size_t n, FILE *stream);

int phenix_runtime_u64_encode(uint64_t value, FILE *stream);

int phenix_runtime_u64_encode_many(const uint64_t *values, size_t n, FILE *stream);

int phenix_runtime_i8_encode(int8_t value, FILE *stream);

int phenix_runtime_i8_encode_many(const int8_t *values, size_t n, FILE *stream);

int phenix_runtime_i16_encode(int16_t value, FILE *stream);

int phenix_runtime_i16_encode_many(const int16_t *values, size_t n, FILE *stream);

int phenix_runtime_i32_encode(int32_t value, FILE *stream);

int phenix_runtime_i32_encode_many(const int32_t *values, size_t n, FILE *stream);

int phenix_runtime_i64_encode(int64_t value, FILE *stream);

int phenix_runtime_i64_encode_many(const int64_t *values, size_t n, FILE *stream);

int phenix_runtime_f32_encode(float value, FILE *stream);

int phenix_runtime_f32_encode_many(const float *values, size_t n, FILE *stream);

int phenix_runtime_f64_encode(double value, FILE *stream);

int phenix_runtime_f64_encode_many(const double *values, size_t n, FILE *stream);

int phenix_runtime_string_encode(const char *value, FILE *stream);

int phenix_runtime_string_encode_many(const char *const *values, size_t n, FILE *stream);

int phenix_runtime_encode_discriminant(size_t n, FILE *stream);

int phenix_runtime_encode_discriminant_relaxed(size_t n, FILE *stream);

#endif /* PHENIX_RUNTIME_H */
