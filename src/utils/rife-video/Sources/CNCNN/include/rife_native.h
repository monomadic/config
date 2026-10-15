#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
void *rv_ncnn_create(const char *model, int padding, int fp16_math, char *error, size_t error_size);
void rv_ncnn_destroy(void *context);
int rv_ncnn_process(void *context, const uint8_t *left, const uint8_t *right,
                    uint8_t *output, int width, int height, float phase);
#ifdef __cplusplus
}
#endif
