#ifndef ENGINE_SRC_CORE_RENDERER_RENDERER_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERER_H_

namespace engine {
namespace renderer {

/**
 * Available rendering APIs to be set in games for the rendering engine to set
 * which graphics context to use at runtime.
 */
enum class RendererAPI {
  None = 0,
  OpenGL = 1,
};

/**
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
