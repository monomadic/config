#include "rife_native.h"
#include "rife.h"
#include "gpu.h"
#include <cstdio>
#include <exception>
#include <memory>
#include <mutex>
#include <stdexcept>

extern "C" void *rv_ncnn_create(const char *model, int padding, int fp16_math, char *error, size_t size) {
    try {
        if (std::string(model).size() > 230) throw std::runtime_error("NCNN model path exceeds upstream 256-byte buffer limit");
        static std::once_flag once;
        std::call_once(once, [] { ncnn::create_gpu_instance(); });
        if (ncnn::get_gpu_count() < 1) throw std::runtime_error("no Vulkan GPU available");
        auto engine = std::make_unique<RIFE>(ncnn::get_default_gpu_index(), false, false, false, 1, false, true, padding);
        if (engine->load(model, fp16_math != 0)) throw std::runtime_error("NCNN model load failed");
        return engine.release();
    } catch (const std::exception& e) { snprintf(error, size, "%s", e.what()); return nullptr; }
}
extern "C" void rv_ncnn_destroy(void *context) { delete static_cast<RIFE *>(context); }
extern "C" int rv_ncnn_process(void *context, const uint8_t *left, const uint8_t *right,
                               uint8_t *output, int width, int height, float phase) {
    try {
        ncnn::Mat a(width, height, const_cast<uint8_t *>(left), (size_t)3, 1);
        ncnn::Mat b(width, height, const_cast<uint8_t *>(right), (size_t)3, 1);
        ncnn::Mat out(width, height, output, (size_t)3, 1);
        return static_cast<RIFE *>(context)->process(a, b, phase, out);
    } catch (...) { return -1; }
}
