#ifndef ENGINE_SRC_CORE_RENDERER_RENDERER_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERER_H_

namespace engine {
namespace renderer {

// Available rendering APIs.
enum class RendererAPI {
  None = 0,
  OpenGL = 1,
};

// The Renderer.
class Renderer {
 public:
  inline static RendererAPI GetAPI() { return kRenderAPI_; }
 private:
  static RendererAPI kRenderAPI_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERER_H_
