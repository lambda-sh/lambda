#ifndef TOOLS_SANDBOX_SRC_SANDBOX2D_H_
#define TOOLS_SANDBOX_SRC_SANDBOX2D_H_

#include <Lambda/Lambda.h>

namespace tools {
namespace sandbox {

// 2D Rendering example layer.
class Sandbox2D : public lambda::core::layers::Layer {
 public:
  Sandbox2D();
  ~Sandbox2D() = default;
  void OnAttach() override;
  void OnDetach() override;
  void OnImGuiRender() override;
  void OnUpdate(lambda::lib::TimeStep) override;
  void OnEvent(lambda::core::events::Event* const event) override;

 private:
  lambda::core::OrthographicCameraController camera_controller_;
  glm::vec4 shader_color_ = {0.8f, 0.3f, 0.2f, 1.0f};

  bool quad_size_increasing_ = true;
  glm::vec2 quad_size_ = { 0.0, 0.0 };

  lambda::core::memory::Shared<lambda::core::renderer::Texture2D>
      checkerboard_texture_;

  lambda::core::memory::Shared<lambda::core::renderer::Shader> shader_;
  lambda::core::memory::Shared<lambda::core::renderer::VertexArray>
      vertex_array_;

  lambda::core::memory::Shared<lambda::core::renderer::VertexBuffer>
      vertex_buffer_;
  lambda::core::memory::Shared<lambda::core::renderer::IndexBuffer>
      index_buffer_;
};

}  // namespace sandbox
}  // namespace tools

#endif  // TOOLS_SANDBOX_SRC_SANDBOX2D_H_
