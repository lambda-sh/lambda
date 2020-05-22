#include "core/renderer/Renderer.h"

namespace engine {
namespace renderer {

// TODO(C3NZ): Update this to not be a hardcoded value and instead one that is
// determined at runtime.
RendererAPI Renderer::kRenderAPI_ = RendererAPI::OpenGL;

}  // namespace renderer
}  // namespace engine
