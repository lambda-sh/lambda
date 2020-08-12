/**
 * @file GraphicsContext.h
 * @brief The Graphics context definition.
 *
 * Platform independent graphics context that is to serve as the basis for
 * initializing and handling the current graphics context.
 */
#ifndef LAMBDA_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_
#define LAMBDA_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_

namespace lambda {
namespace core {
namespace renderer {

class GraphicsContext {
 public:
  virtual void Init() = 0;
  virtual void SwapBuffers() = 0;
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_RENDERER_GRAPHICSCONTEXT_H_

/**
 * @class GraphicsContext
 * @brief The GrahpicsContext base class implementation.
 *
 * In order for a graphics API to be supported within the renderer, it needs to
 * extend and implement the functionality found in here.
 */
