#ifndef ENGINE_FFI_H
#define ENGINE_FFI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Engine Engine;

Engine *engine_init(void *layer_ptr, uint32_t width, uint32_t height);

void engine_frame(Engine *engine, float dt_seconds);

void engine_free(Engine *engine);

#ifdef __cplusplus
}
#endif

#endif /* ENGINE_FFI_H */
