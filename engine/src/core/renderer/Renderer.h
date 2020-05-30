/**
 * @file engine/src/core/renderer/Renderer.h
 * @brief The rendering API.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERER_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERER_H_

namespace engine {
namespace renderer {

/**
 * @enum RendererAPI
 * @brief The graphics APIs that are available through the engine.
 *
 * Available rendering APIs to be set in games for the rendering engine to set
 * which graphics context to use at runtime. Currently only supports OpenGL.
 */
enum class RendererAPI {
  None = 0,
  OpenGL = 1,
};

/**
 * @class Renderer
 * @brief A lightweight rendering API implementation. Allows generalized calls
 * to be written for users
 *
 * A lightweight and not fully finished rendering API that lets you set the a
 * specific graphics context to use for rendering. This must be set externally
 * in any rendering application.
 */
class Renderer {
 public:
  inline static RendererAPI GetAPI() { return kRenderAPI_; }
 private:
  static RendererAPI kRenderAPI_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERER_H_
