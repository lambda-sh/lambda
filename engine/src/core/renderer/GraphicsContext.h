#ifndef ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_
#define ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_

namespace engine {
namespace renderer {

class GraphicsContext {
 public:
  virtual void Init() = 0;
  virtual void SwapBuffers() = 0;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_
