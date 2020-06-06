#include "core/renderer/RenderCommand.h"

#include "platform/opengl/OpenGLRendererAPI.h"

namespace engine {
namespace renderer {

RendererAPI* RenderCommand::renderer_API_ =
    new platform::opengl::OpenGLRendererAPI();

}  // namespace renderer
}  // namespace engine
