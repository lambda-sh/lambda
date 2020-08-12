#include "core/renderer/RenderCommand.h"

#include "platform/opengl/OpenGLRendererAPI.h"

namespace lambda {
namespace core {
namespace renderer {

memory::Unique<RendererAPI> RenderCommand::renderer_API_ =
    memory::CreateUnique<platform::opengl::OpenGLRendererAPI>();

}  // namespace renderer
}  // namesapce core
}  // namespace lambda
