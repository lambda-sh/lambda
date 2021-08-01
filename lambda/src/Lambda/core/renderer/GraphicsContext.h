/// @file GraphicsContext.h
/// @brief The Graphics context definition.
///
/// Platform independent graphics context that is to serve as the basis for
/// initializing and handling the current graphics context.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_GRAPHICSCONTEXT_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_GRAPHICSCONTEXT_H_

namespace lambda::core::renderer {

/// @brief The Graphics context abstraction.
///
/// The renderer requires a working graphics context to be in place in order for
/// graphics to properly be implemented.
class GraphicsContext {
 public:
  virtual ~GraphicsContext() = default;
  virtual void Init() = 0;
  virtual void SwapBuffers() = 0;
};

}  // namespace lambda::core::renderer

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_GRAPHICSCONTEXT_H_
