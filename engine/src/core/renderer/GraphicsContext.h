/**
 * @file engine/src/core/renderer/GraphicsContext.h
 * @brief The Graphics context definition.
 *
 * Platform independent graphics context that is to serve as the basis for
 * initializing and handling the current graphics context.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_
#define ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_

namespace engine {
namespace renderer {

/**
 * @class GraphicsContext
 * @brief The GrahpicsContext base class implementation.
 *
 * In order for a graphics API to be supported within the renderer, it needs to
 * extend and implement the functionality found in here.
 */
class GraphicsContext {
 public:
  virtual void Init() = 0;
  virtual void SwapBuffers() = 0;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_
